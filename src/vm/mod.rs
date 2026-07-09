pub mod gc;
pub mod interpreter;
#[cfg(feature = "jit")]
pub mod jit;
pub mod well_known;

pub use interpreter::{EventSource, Interpreter};
