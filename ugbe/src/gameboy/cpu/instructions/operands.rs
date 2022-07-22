mod deref;
mod general;
mod immediate;
mod instances;
mod registers;

pub use deref::{DerefDecOperand, DerefIncOperand, DerefOperand, DerefOperandToU16};

pub use general::{
    Operand, OperandIn, OperandOut, OperandReadExecution, OperandReadExecutionState,
    OperandWriteExecution, OperandWriteExecutionState,
};

pub use registers::{OperandRegister, ReadRegister, WriteRegister};

pub use immediate::{OperandImmediate, ReadImmediate};

pub use instances::*;
