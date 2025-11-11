mod error;
mod spantrace;
#[cfg(not(target_arch = "wasm32"))]
pub use tracing_error::ErrorLayer;

pub use self::{
    error::{Error, Result, ResultExt},
    spantrace::SpanTrace,
};
