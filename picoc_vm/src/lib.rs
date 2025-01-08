//! Virtual machine of picoc.
//!
//! This machine interprets picoc vm instruction sets.

mod decode;
mod error;
mod opcode;
mod vm;

pub use error::Error;
pub use opcode::Opcode;
pub use vm::PicocVm;
pub use vm::Registers;

pub use vm::VM_STACK_SIZE;
pub use vm::VM_INST_MEMORY_SIZE;
