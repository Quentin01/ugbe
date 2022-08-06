use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::bus::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::operands::{
    Operand, OperandIn, OperandOut, OperandReadExecution, OperandReadExecutionState,
    OperandWriteExecution, OperandWriteExecutionState,
};
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Ld<Dst, Src, const WAIT_ONE_EXTRA_CYCLE: bool = false>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand<Value = <Src as Operand>::Value> + OperandOut + 'static,
{
    phantom: PhantomData<(Dst, Src)>,
}

impl<Dst, Src, const WAIT_ONE_EXTRA_CYCLE: bool> Ld<Dst, Src, WAIT_ONE_EXTRA_CYCLE>
where
    Src: Operand + OperandIn,
    Dst: Operand<Value = <Src as Operand>::Value> + OperandOut,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Dst, Src, const WAIT_ONE_EXTRA_CYCLE: bool> Instruction for Ld<Dst, Src, WAIT_ONE_EXTRA_CYCLE>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand<Value = <Src as Operand>::Value> + OperandOut + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("LD {}, {}", Dst::str(), Src::str()).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(LdExecution::<Dst, Src, WAIT_ONE_EXTRA_CYCLE>::Start(
            PhantomData,
        ))
    }
}

enum LdExecution<Dst, Src, const WAIT_ONE_EXTRA_CYCLE: bool = false>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand<Value = <Src as Operand>::Value> + OperandOut + 'static,
{
    Start(PhantomData<(Dst, Src)>),
    ReadingFromSrc(Box<dyn OperandReadExecution<Src::Value>>),
    WritingToDst(Box<dyn OperandWriteExecution>),
    Complete,
}

impl<Dst, Src, const WAIT_ONE_EXTRA_CYCLE: bool> InstructionExecution
    for LdExecution<Dst, Src, WAIT_ONE_EXTRA_CYCLE>
where
    Src: Operand + OperandIn + 'static,
    Dst: Operand<Value = <Src as Operand>::Value> + OperandOut + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                let _ = std::mem::replace(self, Self::ReadingFromSrc(Src::read_value()));
                self.next(registers, data_bus)
            }
            Self::ReadingFromSrc(mut operand_read_value) => {
                match operand_read_value.next(registers, data_bus) {
                    OperandReadExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::ReadingFromSrc(operand_read_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandReadExecutionState::Complete(value) => {
                        let _ =
                            std::mem::replace(self, Self::WritingToDst(Dst::write_value(value)));
                        self.next(registers, data_bus)
                    }
                }
            }
            Self::WritingToDst(mut operand_write_value) => {
                match operand_write_value.next(registers, data_bus) {
                    OperandWriteExecutionState::Yield(memory_operation) => {
                        let _ = std::mem::replace(self, Self::WritingToDst(operand_write_value));
                        InstructionExecutionState::YieldMemoryOperation(memory_operation)
                    }
                    OperandWriteExecutionState::Complete => {
                        let _ = std::mem::replace(self, Self::Complete);

                        if WAIT_ONE_EXTRA_CYCLE {
                            InstructionExecutionState::YieldMemoryOperation(MemoryOperation::None)
                        } else {
                            self.next(registers, data_bus)
                        }
                    }
                }
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
