use core::num;
use std::{error, io};
use std::fmt::{Display, Formatter};

/// The error type for VM operations.
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    LabelNotFound(String),
    MemoryOutOfBound,
    ParseIntError(num::ParseIntError),
    OpcodeNotFound,
    OperandNotFound,
    StackOverflow,
    StackOutOfBound,
    StackUnderflow,
    UnknownOpcode(String),
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
