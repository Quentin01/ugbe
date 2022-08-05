use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::condition::Condition;
use super::super::operands::{Operand, OperandIn, OperandReadExecution, OperandReadExecutionState};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Jr<Cond, Op>
where
    Cond: Condition + 'static,
    Op: Operand<Value = i8> + OperandIn + 'static,
{
    phantom: PhantomData<(Cond, Op)>,
}

impl<Cond, Op> Jr<Cond, Op>
where
    Cond: Condition + 'static,
    Op: Operand<Value = i8> + OperandIn + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Cond, Op> Instruction for Jr<Cond, Op>
where
    Cond: Condition + 'static,
    Op: Operand<Value = i8> + OperandIn + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        if Cond::STR.len() == 0 {
            format!("JR {}", Op::str()).into()
        } else {
            format!("JR {}, {}", Cond::STR, Op::str()).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(JrExecution::<Cond, Op>::Start(PhantomData))
    }
}

enum JrExecution<Cond, Op>
where
    Cond: Condition + 'static,
    Op: Operand<Value = i8> + OperandIn + 'static,
{
    Start(PhantomData<(Cond, Op)>),
    ReadingOffset(Box<dyn OperandReadExecution<Op::Value>>),
    SettingNewAddress(i8),
    Complete,
}

impl<Cond, Op> InstructionExecution for JrExecution<Cond, Op>
where
    Cond: Condition + 'static,
    Op: Operand<Value = i8> + OperandIn + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::ReadingOffset(Op::read_value()));
                self.next(registers, data_bus)
            }
            Self::ReadingOffset(mut operand_read_value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::ReadingOffset(operand_read_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandReadExecutionState::Complete(value) => {
                        if Cond::check(registers) {
                            let _ = std::mem::replace(self, Self::SettingNewAddress(value));
                        } else {
                            let _ = std::mem::replace(self, Self::Complete);
                        }
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::SettingNewAddress(offset) => {
                let pc = registers.pc();
                registers.set_pc(pc.wrapping_add(offset as u16));

                let _ = std::mem::replace(self, Self::Complete);
                InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
