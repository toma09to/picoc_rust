use core::num;
use std::{error, io};
use std::fmt::{Display, Formatter};

/// The error type for VM operations.
#[derive(Debug)]
pub enum Error {
    /// The error from [`std::io::Error`].
    ///
    /// This error is raised when an I/O error(e.g. File Not Found, Permission denied) occurs.
    IoError(io::Error),
    /// Unknown label is found in an operand.
    LabelNotFound(String),
    /// The value of PC exceeds an instruction memory.
    MemoryOutOfBound,
    /// The error from [`std::num::ParseIntError`].
    ///
    /// VM cannot parse an integer operand.
    ParseIntError(num::ParseIntError),
    /// An opcode is not found.
    OpcodeNotFound,
    /// An operand is not found.
    OperandNotFound,
    /// The value of SP exceeds the top of a stack (SP < 0).
    StackOverflow,
    /// VM attempts to read outside of a stack.
    StackOutOfBound,
    /// The value of SP exceeds the bottom of a stack (SP >= [`VM_STACK_SIZE`](crate::VM_STACK_SIZE)).
    StackUnderflow,
    /// An unknown opcode is found.
    UnknownOpcode(String),
    /// VM halted.
    VmHalted,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(err) => err.fmt(f),
            Error::ParseIntError(err) => err.fmt(f),
            Error::LabelNotFound(name) => write!(f, "Label '{}' is not found", name),
            Error::MemoryOutOfBound => write!(f, "PC out of bounds"),
            Error::OpcodeNotFound => write!(f, "Opcode is not found"),
            Error::OperandNotFound => write!(f, "Operand is not found"),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::StackOutOfBound => write!(f, "SP out of bounds"),
            Error::StackUnderflow => write!(f, "Stack underflow"),
            Error::UnknownOpcode(name) => write!(f, "Unknown opcode '{}' is found", name),
            Error::VmHalted => write!(f, "VM is already halted"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Error::IoError(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Error::ParseIntError(error)
    }
}
