use std::borrow::Cow;

use crate::gameboy::cpu::CpuOperation;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EI {}

impl EI {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Instruction for EI {
    fn raw_desc(&self) -> Cow<'static, str> {
        "EI".into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(EIExecution::Start)
    }
}

enum EIExecution {
    Start,
    Complete,
}

impl InstructionExecution for EIExecution {
    fn next(&mut self, _: &mut Registers, _: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start => {
                let _ = std::mem::replace(self, Self::Complete);
                InstructionExecutionState::YieldCpuOperation(CpuOperation::EnableInterrupt)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
