use std::borrow::Cow;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Invalid<const OPCODE: u8, const IS_CB_PREFIXED: bool> {}

impl<const OPCODE: u8, const IS_CB_PREFIXED: bool> Invalid<OPCODE, IS_CB_PREFIXED> {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<const OPCODE: u8, const IS_CB_PREFIXED: bool> Instruction for Invalid<OPCODE, IS_CB_PREFIXED> {
    fn raw_desc(&self) -> Cow<'static, str> {
        if IS_CB_PREFIXED {
            format!("INVALID 0xCB 0x{:02X}", OPCODE).into()
        } else {
            format!("INVALID 0x{:02X}", OPCODE).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(InvalidExecution::<OPCODE, IS_CB_PREFIXED> {})
    }
}

struct InvalidExecution<const OPCODE: u8, const IS_CB_PREFIXED: bool> {}

impl<const OPCODE: u8, const IS_CB_PREFIXED: bool> InstructionExecution
    for InvalidExecution<OPCODE, IS_CB_PREFIXED>
{
    fn next(&mut self, registers: &mut Registers, _: u8) -> InstructionExecutionState {
        if IS_CB_PREFIXED {
            panic!(
                "Invalid instruction encountered 0xCB 0x{:02X} at ${:04x}",
                OPCODE,
                registers.pc().wrapping_sub(2)
            )
        } else {
            panic!(
                "Invalid instruction encountered 0x{:02X} at ${:04x}",
                OPCODE,
                registers.pc().wrapping_sub(1)
            )
        }
    }
}
