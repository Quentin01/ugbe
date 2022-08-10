use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::condition::Condition;
use super::super::operands::{Operand, OperandIn, OperandReadExecution, OperandReadExecutionState};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Call<Cond, Op>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    phantom: PhantomData<(Cond, Op)>,
}

impl<Cond, Op> Call<Cond, Op>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Cond, Op> Instruction for Call<Cond, Op>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        if Cond::STR.len() == 0 {
            format!("CALL {}", Op::str()).into()
        } else {
            format!("CALL {}, {}", Cond::STR, Op::str()).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(CallExecution::<Cond, Op>::Start(PhantomData))
    }
}

enum CallExecution<Cond, Op>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    Start(PhantomData<(Cond, Op)>),
    ReadingAddress(Box<dyn OperandReadExecution<Op::Value> + 'static>),
    DecrementingSP(u16),
    PushingMsbPC(u16),
    PushingLsbPC(u16),
    ChangingPC(u16),
    Complete,
}

impl<Cond, Op> InstructionExecution for CallExecution<Cond, Op>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::ReadingAddress(Op::read_value()));
                self.next(registers, data_bus)
            }
            Self::ReadingAddress(mut operand_read_value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::ReadingAddress(operand_read_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandReadExecutionState::Complete(value) => {
                        if Cond::check(registers) {
                            let _ = std::mem::replace(self, Self::DecrementingSP(value));
                        } else {
                            let _ = std::mem::replace(self, Self::Complete);
                        }

                        self.next(registers, data_bus)
                    }
                }
            }
            Self::DecrementingSP(new_address) => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let _ = std::mem::replace(self, Self::PushingMsbPC(new_address));
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
            }
            Self::PushingMsbPC(new_address) => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let [value, _] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::PushingLsbPC(new_address));
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::PushingLsbPC(new_address) => {
                let sp = registers.sp();

                let [_, value] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::ChangingPC(new_address));
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::ChangingPC(new_address) => {
                registers.set_pc(new_address);

                let _ = std::mem::replace(self, Self::Complete);
                self.next(registers, data_bus)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
