use color_backtrace::Verbosity;
use snafu::GenerateImplicitData;
use tracing_error::SpanTraceStatus;

pub type TestResult<A = (), E = TestError> = std::result::Result<A, E>;

pub trait TestResultExt<T> {
    #[track_caller]
    fn context<C>(self, context: C) -> Result<T, TestError>
    where
        C: AsRef<str>;
}

impl<T, E> TestResultExt<T> for Result<T, E>
where
    E: snafu::Error + Sync + Send + 'static,
{
    #[track_caller]
    fn context<C>(self, context: C) -> Result<T, TestError>
    where
        C: AsRef<str>,
    {
        match self {
            Ok(v) => Ok(v),
            Err(error) => Err(TestError::Message {
                message: context.as_ref().into(),
                span_trace: GenerateImplicitData::generate(),
                source: Box::new(error),
                backtrace: GenerateImplicitData::generate(),
            }),
        }
    }
}

// Trait safe version
pub trait Formatted: snafu::Error {
    /// Returns a [`Backtrace`][] that may be printed.
    fn backtrace(&self) -> Option<Backtrace<'_>>;
}

impl<T: snafu::Error + snafu::ErrorCompat> Formatted for T {
    fn backtrace(&self) -> Option<Backtrace<'_>> {
        snafu::ErrorCompat::backtrace(&*self).map(Backtrace::Crate)
    }
}

pub enum TestError {
    Source {
        source: Box<dyn Formatted>,
        span_trace: SpanTrace,
        backtrace: Option<snafu::Backtrace>,
    },
    Message {
        message: String,
        span_trace: SpanTrace,
        source: Box<dyn snafu::Error>,
        backtrace: Option<snafu::Backtrace>,
    },
    Anyhow {
        source: anyhow::Error,
        span_trace: SpanTrace,
        backtrace: Option<snafu::Backtrace>,
    },
}

impl<E1: Formatted + Send + Sync + 'static> From<E1> for TestError {
    fn from(value: E1) -> Self {
        Self::Source {
            source: Box::new(value),
            span_trace: GenerateImplicitData::generate(),
            backtrace: GenerateImplicitData::generate(),
        }
    }
}

impl std::fmt::Debug for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let verb = Verbosity::from_env();

        let filters = [
            "<n0_snafu::testerror::TestError",
            "n0_snafu::testerror::TestError::anyhow",
            "<core::pin::Pin<P> as core::future::future::Future>::poll",
            "<core::result::Result<T,F> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible,E>>>::from_residual",
        ];

        let mut printer =
            color_backtrace::BacktracePrinter::new().add_frame_filter(Box::new(move |frames| {
                frames.retain(|frame| {
                    frame
                        .name
                        .as_ref()
                        .map(|name| {
                            for f in &filters {
                                if name.starts_with(f) {
                                    return false;
                                }
                            }
                            true
                        })
                        .unwrap_or(true)
                })
            }));

        if verb != Verbosity::Full {
            printer = printer.add_frame_filter(Box::new(|frames| {
                frames.retain(|frame| !frame.is_dependency_code())
            }));
        }

        let stack = self.stack();

        writeln!(f)?;
        for (i, (_, source)) in stack.iter().skip(1).enumerate() {
            match source {
                Source::Root => {}
                _ => writeln!(f, "    {}: {}", i, source)?,
            }
        }

        writeln!(f)?;

        // Span Trace
        if self.span_trace().status() == SpanTraceStatus::CAPTURED {
            writeln!(f, "Span trace:")?;
            writeln!(f, "{}\n", self.span_trace())?;
        }

        // Backtrace
        let empty_bt = snafu::Backtrace::from(Vec::new());
        for (bt, _) in stack.into_iter() {
            let bt = bt.unwrap_or(Backtrace::Crate(&empty_bt));
            let s = printer.format_trace_to_string(&bt).unwrap();
            writeln!(f, "\n{}", s)?;
        }

        Ok(())
    }
}

impl TestError {
    pub fn span_trace(&self) -> &SpanTrace {
        match self {
            Self::Source { span_trace, .. } => span_trace,
            Self::Message { span_trace, .. } => span_trace,
            Self::Anyhow { span_trace, .. } => span_trace,
        }
    }

    pub fn anyhow(err: anyhow::Error) -> Self {
        Self::Anyhow {
            source: err,
            span_trace: GenerateImplicitData::generate(),
            backtrace: GenerateImplicitData::generate(),
        }
    }

    pub fn stack(&self) -> Vec<(Option<Backtrace>, Source<'_>)> {
        let mut traces = Vec::new();

        match self {
            Self::Source {
                source, backtrace, ..
            } => {
                // current trace
                traces.push((backtrace.as_ref().map(Backtrace::Crate), Source::Root));
                traces.push((source.backtrace(), Source::Formatted(source.as_ref())));

                // collect the traces from our sources
                let mut source = source.source();

                while let Some(s) = source {
                    if let Some(this) = s.downcast_ref::<&dyn Formatted>() {
                        traces.push((this.backtrace(), Source::Formatted(*this)));
                    } else {
                        traces.push((None, Source::Error(s)));
                    }
                    source = s.source();
                }
            }
            Self::Message {
                source, backtrace, ..
            } => {
                // current trace
                traces.push((backtrace.as_ref().map(Backtrace::Crate), Source::Root));

                // collect the traces from our sources
                let mut source = Some(source.as_ref());

                while let Some(s) = source {
                    if let Some(this) = s.downcast_ref::<&dyn Formatted>() {
                        traces.push((this.backtrace(), Source::Formatted(*this)));
                    } else {
                        traces.push((None, Source::Error(s)));
                    }
                    source = s.source();
                }
            }
            Self::Anyhow {
                source, backtrace, ..
            } => {
                // current trace
                traces.push((backtrace.as_ref().map(Backtrace::Crate), Source::Root));

                traces.push((
                    Some(Backtrace::Std(source.backtrace())),
                    Source::Anyhow(source),
                ));

                for s in source.chain().skip(1) {
                    if let Some(this) = s.downcast_ref::<&dyn Formatted>() {
                        traces.push((this.backtrace(), Source::Formatted(*this)));
                    } else {
                        traces.push((None, Source::Error(s)));
                    }
                }
            }
        }

        traces
    }
}

#[derive(Clone)]
pub enum Backtrace<'a> {
    Crate(&'a snafu::Backtrace),
    Std(&'a std::backtrace::Backtrace),
}

impl color_backtrace::Backtrace for Backtrace<'_> {
    fn frames(&self) -> Vec<color_backtrace::Frame> {
        match self {
            Self::Crate(bt) => color_backtrace::Backtrace::frames(*bt),
            Self::Std(bt) => {
                // no comment, things are sad in std land
                let parsed_bt = btparse::deserialize(bt).expect("failed to parse stacks");
                color_backtrace::Backtrace::frames(&parsed_bt)
            }
        }
    }
}

pub enum Source<'a> {
    Root,
    Formatted(&'a dyn Formatted),
    Error(&'a dyn snafu::Error),
    Anyhow(&'a anyhow::Error),
}

impl core::fmt::Display for Source<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Root => write!(f, "Root"),
            Self::Formatted(e) => e.fmt(f),
            Self::Error(e) => e.fmt(f),
            Self::Anyhow(e) => e.fmt(f),
        }
    }
}

impl snafu::ErrorCompat for TestError {
    fn backtrace(&self) -> Option<&snafu::Backtrace> {
        self.stack().last().and_then(|(bt, _)| match *bt {
            Some(Backtrace::Crate(bt)) => Some(bt),
            _ => None,
        })
    }
}

impl core::fmt::Display for TestError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Source { source, .. } => {
                write!(f, "{}", source)
            }
            Self::Message {
                message, source, ..
            } => {
                write!(f, "{}: {}", message, source)
            }
            Self::Anyhow { source, .. } => source.fmt(f),
        }
    }
}

pub struct SpanTrace(tracing_error::SpanTrace);

impl std::fmt::Debug for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for SpanTrace {
    type Target = tracing_error::SpanTrace;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl snafu::GenerateImplicitData for SpanTrace {
    fn generate() -> Self {
        Self(tracing_error::SpanTrace::capture())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anyhow_compat() -> TestResult {
        fn ok() -> anyhow::Result<()> {
            Ok(())
        }

        ok().map_err(TestError::anyhow)?;

        Ok(())
    }
}
