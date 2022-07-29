use std::borrow::Cow;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Rst<const ADDRESS: u8> {}

impl<const ADDRESS: u8> Rst<ADDRESS> {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<const ADDRESS: u8> Instruction for Rst<ADDRESS> {
    fn raw_desc(&self) -> Cow<'static, str> {
        format!("RST ${:02x}", ADDRESS).into()
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(RstExecution::<ADDRESS>::Start)
    }
}

enum RstExecution<const ADDRESS: u8> {
    Start,
    DecrementingSP,
    PushingMsbPC,
    PushingLsbPC,
    ChangingPC,
    Complete,
}

impl<const ADDRESS: u8> InstructionExecution for RstExecution<ADDRESS> {
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start => {
                let _ = std::mem::replace(self, Self::DecrementingSP);
                self.next(registers, data_bus)
            }
            Self::DecrementingSP => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let _ = std::mem::replace(self, Self::PushingMsbPC);
                InstructionExecutionState::Yield(MemoryOperation::None)
            }
            Self::PushingMsbPC => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let [value, _] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::PushingLsbPC);
                InstructionExecutionState::Yield(MemoryOperation::Write { address: sp, value })
            }
            Self::PushingLsbPC => {
                let sp = registers.sp();

                let [_, value] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::ChangingPC);
                InstructionExecutionState::Yield(MemoryOperation::Write { address: sp, value })
            }
            Self::ChangingPC => {
                registers.set_pc(ADDRESS as u16);

                let _ = std::mem::replace(self, Self::Complete);
                self.next(registers, data_bus)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
