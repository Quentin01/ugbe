use std::marker::PhantomData;

use super::super::super::registers::Registers;
use super::{
    Operand, OperandReadExecution, OperandReadExecutionState, OperandWriteExecution,
    OperandWriteExecutionState,
};

pub trait OperandRegister: Operand {
    fn read_register(registers: &mut Registers) -> <Self as Operand>::Value;
    fn write_register(registers: &mut Registers, value: <Self as Operand>::Value);
}

pub struct ReadRegister<Op>
where
    Op: OperandRegister,
{
    phantom: PhantomData<Op>,
}

impl<Op> ReadRegister<Op>
where
    Op: OperandRegister,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Op> OperandReadExecution<Op::Value> for ReadRegister<Op>
where
    Op: OperandRegister,
{
    fn next(&mut self, registers: &mut Registers, _: u8) -> OperandReadExecutionState<Op::Value> {
        OperandReadExecutionState::<Op::Value>::Complete(Op::read_register(registers))
    }
}

pub struct WriteRegister<Op>
where
    Op: OperandRegister,
{
    value: <Op as Operand>::Value,
}

impl<Op> WriteRegister<Op>
where
    Op: OperandRegister,
{
    pub const fn new(value: <Op as Operand>::Value) -> Self {
        Self { value }
    }
}

impl<Op> OperandWriteExecution for WriteRegister<Op>
where
    Op: OperandRegister,
{
    fn next(&mut self, registers: &mut Registers, _: u8) -> OperandWriteExecutionState {
        Op::write_register(registers, self.value);
        OperandWriteExecutionState::Complete
    }
}
