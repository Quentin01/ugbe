use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::operands::{
    Operand, OperandOut, OperandWriteExecution, OperandWriteExecutionState,
};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Pop<Op>
where
    Op: Operand<Value = u16> + OperandOut + 'static,
{
    phantom: PhantomData<Op>,
}

impl<Op> Pop<Op>
where
    Op: Operand<Value = u16> + OperandOut + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Op> Instruction for Pop<Op>
where
    Op: Operand<Value = u16> + OperandOut + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("POP {}", Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(PopExecution::<Op>::Start(PhantomData))
    }
}

enum PopExecution<Op>
where
    Op: Operand<Value = u16> + OperandOut + 'static,
{
    Start(PhantomData<Op>),
    PopingLsb,
    PopingMsb,
    PoppingEnd(u8),
    WritingValue(Box<dyn OperandWriteExecution>),
    Complete,
}

impl<Op> InstructionExecution for PopExecution<Op>
where
    Op: Operand<Value = u16> + OperandOut + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::PopingLsb);
                self.next(registers, data_bus)
            }
            Self::PopingLsb => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_add(1));

                let _ = std::mem::replace(self, Self::PopingMsb);
                InstructionExecutionState::Yield(MemoryOperation::Read { address: sp })
            }
            Self::PopingMsb => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_add(1));

                let _ = std::mem::replace(self, Self::PoppingEnd(data_bus));
                InstructionExecutionState::Yield(MemoryOperation::Read { address: sp })
            }
            Self::PoppingEnd(lsb) => {
                let value = u16::from_be_bytes([data_bus, lsb]);

                let _ = std::mem::replace(self, Self::WritingValue(Op::write_value(value)));
                self.next(registers, data_bus)
            }

            Self::WritingValue(mut operand_write_value) => {
                match operand_write_value.next(registers, data_bus) {
                    OperandWriteExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::WritingValue(operand_write_value));
                        InstructionExecutionState::Yield(memory_operation)
                    }
                    OperandWriteExecutionState::Complete => {
                        let _ = std::mem::replace(self, Self::Complete);
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
