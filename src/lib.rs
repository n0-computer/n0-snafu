// pub mod report;
// pub use self::report::Report;
mod once_bool;

mod testerror;
pub use self::testerror::{fmt_err, TestError, TestResult, TestResultExt};
