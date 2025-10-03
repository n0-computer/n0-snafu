mod error;
mod spantrace;
pub use tracing_error::ErrorLayer;

pub use self::{
    error::{Error, Result, ResultExt},
    spantrace::SpanTrace,
};

#[cfg(test)]
mod tests {
    use super::*;
    use nested_enum_utils::common_fields;
    use snafu::{Backtrace, ErrorCompat, ResultExt, Snafu};

    #[common_fields({
    backtrace: Option<Backtrace>,
    #[snafu(implicit)]
    span_trace: SpanTrace,
})]
    #[allow(missing_docs)]
    #[derive(Debug, Snafu)]
    #[non_exhaustive]
    pub enum EnumNoTransparent {
        #[snafu(display("This error wraps another enum"))]
        WrapAnotherEnum { source: AnotherEnumError },
        #[snafu(display("This error wraps a struct"))]
        WrapAStruct { source: StructError },
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("This is a struct error"))]
    pub struct StructError {
        backtrace: Option<Backtrace>,
    }

    #[common_fields({
    backtrace: Option<Backtrace>,
    #[snafu(implicit)]
    span_trace: SpanTrace,
})]
    #[allow(missing_docs)]
    #[derive(Debug, Snafu)]
    #[non_exhaustive]
    pub enum EnumTransparent {
        #[snafu(transparent)]
        #[allow(dead_code)]
        TransparentStruct { source: StructError },
    }

    #[common_fields({
    backtrace: Option<Backtrace>,
    #[snafu(implicit)]
    span_trace: SpanTrace,
})]
    #[allow(missing_docs)]
    #[derive(Debug, Snafu)]
    #[non_exhaustive]
    pub enum AnotherEnumError {
        #[snafu(display("This is a variant error in another enum"))]
        ErrorInAnotherEnum {},
    }

    fn enum_no_transparent_another_enum() -> std::result::Result<(), EnumNoTransparent> {
        another_enum().context(WrapAnotherEnumSnafu)?;
        Ok(())
    }

    fn enum_no_transparent_struct() -> std::result::Result<(), EnumNoTransparent> {
        struct_error().context(WrapAStructSnafu)?;
        Ok(())
    }

    fn enum_transparent_struct() -> std::result::Result<(), EnumTransparent> {
        struct_error()?;
        Ok(())
    }

    fn struct_error() -> std::result::Result<(), StructError> {
        let err = StructSnafu {}.build();
        println!("STRUCT BACKTRACE{:?}", err.backtrace());
        Err(err)
    }

    fn another_enum() -> std::result::Result<(), AnotherEnumError> {
        Err(ErrorInAnotherEnumSnafu {}.build())
    }

    #[test]
    fn test_another_enum() -> Result<()> {
        enum_no_transparent_another_enum()?;
        Ok(())
    }

    #[test]
    fn test_wrap_a_struct() -> Result<()> {
        enum_no_transparent_struct()?;
        Ok(())
    }

    #[test]
    fn test_transparent() -> Result<()> {
        enum_transparent_struct()?;
        Ok(())
    }
}
