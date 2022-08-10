use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::condition::Condition;
use super::super::operands::{Operand, OperandIn, OperandReadExecution, OperandReadExecutionState};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Jp<Cond, Op, const WAIT_ONE_EXTRA_CYCLE: bool = true>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    phantom: PhantomData<(Cond, Op)>,
}

impl<Cond, Op, const WAIT_ONE_EXTRA_CYCLE: bool> Jp<Cond, Op, WAIT_ONE_EXTRA_CYCLE>
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

impl<Cond, Op, const WAIT_ONE_EXTRA_CYCLE: bool> Instruction for Jp<Cond, Op, WAIT_ONE_EXTRA_CYCLE>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        if Cond::STR.len() == 0 {
            format!("JP {}", Op::str()).into()
        } else {
            format!("JP {}, {}", Cond::STR, Op::str()).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(JpExecution::<Cond, Op, WAIT_ONE_EXTRA_CYCLE>::Start(
            PhantomData,
        ))
    }
}

enum JpExecution<Cond, Op, const WAIT_ONE_EXTRA_CYCLE: bool = false>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
{
    Start(PhantomData<(Cond, Op)>),
    ReadingOffset(Box<dyn OperandReadExecution<Op::Value> + 'static>),
    SettingPC(u16),
    Complete,
}

impl<Cond, Op, const WAIT_ONE_EXTRA_CYCLE: bool> InstructionExecution
    for JpExecution<Cond, Op, WAIT_ONE_EXTRA_CYCLE>
where
    Cond: Condition + Send + Sync + 'static,
    Op: Operand<Value = u16> + OperandIn + Send + Sync + 'static,
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
                            let _ = std::mem::replace(self, Self::SettingPC(value));
                        } else {
                            let _ = std::mem::replace(self, Self::Complete);
                        }
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::SettingPC(value) => {
                let pc = registers.pc();
                registers.set_pc(value);

                let _ = std::mem::replace(self, Self::Complete);

                if WAIT_ONE_EXTRA_CYCLE {
                    InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
                } else {
                    self.next(registers, data_bus)
                }
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
