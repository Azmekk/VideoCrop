//! Runtime: processes, files, network. Everything here only *executes* what `crate::core`
//! describes; argument construction and parsing stay in core.

pub mod encoder;
pub mod hwdetect;
pub mod process;
pub mod protocol;
pub mod session;
#[cfg(test)]
mod smoke;
pub mod stream;
pub mod toolcheck;
pub mod tools;
