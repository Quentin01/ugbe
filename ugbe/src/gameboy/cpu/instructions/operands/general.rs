use std::borrow::Cow;

use super::super::super::registers::Registers;

pub trait Operand: Send + Sync {
    type Value: Copy + Send + Sync + 'static;

    fn str() -> Cow<'static, str>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperandReadExecutionState<Value>
where
    Value: Send + Sync + 'static,
{
    Yield(super::super::MemoryOperation),
    Complete(Value),
}

pub trait OperandReadExecution<Value>: Send + Sync + Sync
where
    Value: Send + Sync + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8)
        -> OperandReadExecutionState<Value>;
}

pub trait OperandIn: Operand {
    fn read_value() -> Box<dyn OperandReadExecution<Self::Value> + 'static>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperandWriteExecutionState {
    Yield(super::super::MemoryOperation),
    Complete,
}

pub trait OperandWriteExecution: Send + Sync {
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> OperandWriteExecutionState;
}

pub trait OperandOut: Operand {
    fn write_value(value: Self::Value) -> Box<dyn OperandWriteExecution + 'static>;
}
