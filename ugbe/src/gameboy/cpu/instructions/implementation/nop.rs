use std::borrow::Cow;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nop {}

impl Nop {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Instruction for Nop {
    fn raw_desc(&self) -> Cow<'static, str> {
        "NOP".into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(NopExecution {})
    }
}

struct NopExecution {}

impl InstructionExecution for NopExecution {
    fn next(&mut self, _: &mut Registers, _: u8) -> InstructionExecutionState {
        InstructionExecutionState::Complete
    }
}
