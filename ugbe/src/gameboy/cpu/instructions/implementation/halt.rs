use std::borrow::Cow;

use crate::gameboy::cpu::CpuOperation;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Halt {}

impl Halt {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Instruction for Halt {
    fn raw_desc(&self) -> Cow<'static, str> {
        "HALT".into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(HaltExecution::Start)
    }
}

enum HaltExecution {
    Start,
    Complete,
}

impl InstructionExecution for HaltExecution {
    fn next(&mut self, _: &mut Registers, _: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start => {
                let _ = std::mem::replace(self, Self::Complete);
                InstructionExecutionState::YieldCpuOperation(CpuOperation::Halt)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
