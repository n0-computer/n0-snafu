#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct SpanTrace(tracing_error::SpanTrace);

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct SpanTrace; // Empty struct for WASM

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Debug for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Debug for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpanTrace(wasm - not available)")
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for SpanTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpanTrace not available on WASM")
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::Deref for SpanTrace {
    type Target = tracing_error::SpanTrace;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl snafu::GenerateImplicitData for SpanTrace {
    fn generate() -> Self {
        Self(tracing_error::SpanTrace::capture())
    }
}

#[cfg(target_arch = "wasm32")]
impl snafu::GenerateImplicitData for SpanTrace {
    fn generate() -> Self {
        Self // No-op for WASM
    }
}
