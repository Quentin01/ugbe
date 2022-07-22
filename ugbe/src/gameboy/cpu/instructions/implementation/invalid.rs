use std::borrow::Cow;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Invalid<const Opcode: u8, const CbPrefixed: bool> {}

impl<const Opcode: u8, const CbPrefixed: bool> Invalid<Opcode, CbPrefixed> {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<const Opcode: u8, const CbPrefixed: bool> Instruction for Invalid<Opcode, CbPrefixed> {
    fn raw_desc(&self) -> Cow<'static, str> {
        if CbPrefixed {
            format!("INVALID 0xCB 0x{:02X}", Opcode).into()
        } else {
            format!("INVALID 0x{:02X}", Opcode).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(InvalidExecution::<Opcode, CbPrefixed> {})
    }
}

struct InvalidExecution<const Opcode: u8, const CbPrefixed: bool> {}

impl<const Opcode: u8, const CbPrefixed: bool> InstructionExecution
    for InvalidExecution<Opcode, CbPrefixed>
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        if CbPrefixed {
            panic!(
                "Invalid instruction encountered 0xCB 0x{:02X} at ${:04x}",
                Opcode,
                registers.pc().wrapping_sub(2)
            )
        } else {
            panic!(
                "Invalid instruction encountered 0x{:02X} at ${:04x}",
                Opcode,
                registers.pc().wrapping_sub(1)
            )
        }
    }
}
