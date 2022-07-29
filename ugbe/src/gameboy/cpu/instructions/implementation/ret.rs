use std::borrow::Cow;
use std::marker::PhantomData;

use crate::gameboy::cpu::MemoryOperation;

use super::super::super::registers::Registers;
use super::super::condition::Condition;
use super::super::{Instruction, InstructionExecution, InstructionExecutionState};

pub struct Ret<Cond, const ENABLE_INTERRUPT: bool = false>
where
    Cond: Condition + 'static,
{
    phantom: PhantomData<Cond>,
}

impl<Cond, const ENABLE_INTERRUPT: bool> Ret<Cond, ENABLE_INTERRUPT>
where
    Cond: Condition + 'static,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Cond, const ENABLE_INTERRUPT: bool> Instruction for Ret<Cond, ENABLE_INTERRUPT>
where
    Cond: Condition + 'static,
{
    fn raw_desc(&self) -> Cow<'static, str> {
        if Cond::STR.len() == 0 {
            "RET".into()
        } else {
            format!("RET {}", Cond::STR).into()
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static> {
        Box::new(RetExecution::<Cond, ENABLE_INTERRUPT>::Start(PhantomData))
    }
}

enum RetExecution<Cond, const ENABLE_INTERRUPT: bool = false>
where
    Cond: Condition + 'static,
{
    Start(PhantomData<Cond>),
    WaitingOneCycle,
    PopingLsbPC,
    PopingMsbPC,
    SettingPC(u8),
    Complete,
}

impl<Cond, const ENABLE_INTERRUPT: bool> InstructionExecution
    for RetExecution<Cond, ENABLE_INTERRUPT>
where
    Cond: Condition + 'static,
{
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState {
        match std::mem::replace(self, Self::Complete) {
            Self::Start(_) => {
                if Cond::STR.len() == 0 {
                    let _ = std::mem::replace(self, Self::PopingLsbPC);
                } else {
                    let _ = std::mem::replace(self, Self::WaitingOneCycle);
                }

                self.next(registers, data_bus)
            }
            Self::WaitingOneCycle => {
                if Cond::check(registers) {
                    let _ = std::mem::replace(self, Self::PopingLsbPC);
                } else {
                    let _ = std::mem::replace(self, Self::Complete);
                }

                InstructionExecutionState::Yield(MemoryOperation::None)
            }
            Self::PopingLsbPC => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_add(1));

                let _ = std::mem::replace(self, Self::PopingMsbPC);
                InstructionExecutionState::Yield(MemoryOperation::Read { address: sp })
            }
            Self::PopingMsbPC => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_add(1));

                let _ = std::mem::replace(self, Self::SettingPC(data_bus));
                InstructionExecutionState::Yield(MemoryOperation::Read { address: sp })
            }
            Self::SettingPC(lsb) => {
                let pc = u16::from_be_bytes([data_bus, lsb]);
                registers.set_pc(pc);

                if ENABLE_INTERRUPT {
                    // TODO: Set IME
                }

                let _ = std::mem::replace(self, Self::Complete);
                InstructionExecutionState::Yield(MemoryOperation::None)
            }
            Self::Complete => InstructionExecutionState::Complete,
        }
    }
}
