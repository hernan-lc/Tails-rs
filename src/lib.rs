#![warn(unsafe_op_in_unsafe_fn)]

pub mod cli;
pub mod compiler;
pub mod dotenv;
pub mod errors;
pub mod ffi;
pub mod objects;
pub mod runtime_env;
pub mod vm;

pub use vm::well_known;

mod runtime;

pub use errors::{Error, Result};
pub use objects::Value;
pub use runtime::{RuntimeConfig, TailsRuntime};
