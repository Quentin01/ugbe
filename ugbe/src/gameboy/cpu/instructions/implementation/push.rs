use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::operands::{Operand, OperandIn, OperandReadExecution, OperandReadExecutionState};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Push<Op>
where
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    phantom: PhantomData<Op>,
}

impl<Op> Push<Op>
where
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Op> Instruction for Push<Op>
where
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("PUSH {}", Op::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(PushExecution::<Op>::Start(PhantomData))
    }
}

enum PushExecution<Op>
where
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    Start(PhantomData<Op>),
    ReadingValue(Box<dyn OperandReadExecution<Op::Value> + 'static>),
    DecrementingSP([u8; 2]),
    PushingMsb([u8; 2]),
    PushingLsb([u8; 2]),
    Complete,
}

impl<Op> InstructionExecution for PushExecution<Op>
where
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::ReadingValue(Op::read_value()));
                self.next(registers, data_bus)
            }
            Self::ReadingValue(mut operand_read_value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::ReadingValue(operand_read_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandReadExecutionState::Complete(value) => {
                        let _ = std::mem::replace(self, Self::DecrementingSP(value.to_be_bytes()));
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::DecrementingSP(bytes) => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let _ = std::mem::replace(self, Self::PushingMsb(bytes));
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
            }
            Self::PushingMsb(bytes) => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let [value, _] = bytes;

                let _ = std::mem::replace(self, Self::PushingLsb(bytes));
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::PushingLsb(bytes) => {
                let sp = registers.sp();

                let [_, value] = bytes;

                let _ = std::mem::replace(self, Self::Complete);
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
