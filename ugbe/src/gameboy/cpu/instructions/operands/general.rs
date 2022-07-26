use std::borrow::Cow;

use super::super::super::registers::Registers;

pub trait Operand {
    type Value: Copy;

    fn str() -> Cow<'static, str>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperandReadExecutionState<Value> {
    Yield(super::super::MemoryOperation),
    Complete(Value),
}

pub trait OperandReadExecution<Value> {
    fn next(&mut self, registers: &mut Registers, data_bus: u8)
        -> OperandReadExecutionState<Value>;
}

pub trait OperandIn: Operand {
    fn read_value() -> Box<dyn OperandReadExecution<Self::Value>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperandWriteExecutionState {
    Yield(super::super::MemoryOperation),
    Complete,
}

pub trait OperandWriteExecution {
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> OperandWriteExecutionState;
}

pub trait OperandOut: Operand {
    fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution>;
}
