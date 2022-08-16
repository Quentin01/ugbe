use super::{
    bus::{Bus, MemoryOperation},
    components::{InterruptKind, InterruptLine},
};

mod instructions;
mod registers;

use registers::Registers;

enum State {
    NotStarted,
    WaitingPrefetchRead(bool),
    DecodeAndExec(u8, bool),
    ExecutingInstruction(
        &'static dyn instructions::Instruction,
        Box<dyn instructions::InstructionExecution + 'static>,
    ),
    StartInterruptDispatch,
    InterruptDispatching(InterruptDispatchState),
    AfterHalt,
    Halted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InterruptDispatchState {
    Start,
    DecrementingSP,
    PushingMsbPC,
    PushingLsbPC,
    ChangingPC,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InterruptDispatchExecutionState {
    Yield(MemoryOperation),
    Complete,
}

impl InterruptDispatchState {
    fn next(
        &mut self,
        registers: &mut Registers,
        interrupt_line: &mut dyn InterruptLine,
    ) -> InterruptDispatchExecutionState {
        match std::mem::replace(self, Self::Complete) {
            InterruptDispatchState::Start => {
                let _ = std::mem::replace(self, Self::DecrementingSP);
                InterruptDispatchExecutionState::Yield(MemoryOperation::None)
            }
            InterruptDispatchState::DecrementingSP => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let _ = std::mem::replace(self, Self::PushingMsbPC);
                InterruptDispatchExecutionState::Yield(MemoryOperation::None)
            }
            Self::PushingMsbPC => {
                let sp = registers.sp();
                registers.set_sp(sp.wrapping_sub(1));

                let [value, _] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::PushingLsbPC);
                InterruptDispatchExecutionState::Yield(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::PushingLsbPC => {
                let sp = registers.sp();

                let [_, value] = registers.pc().to_be_bytes();

                let _ = std::mem::replace(self, Self::ChangingPC);
                InterruptDispatchExecutionState::Yield(MemoryOperation::Write {
                    address: sp,
                    value,
                })
            }
            Self::ChangingPC => {
                let interrupt = interrupt_line.highest_priority();
                if let Some(interrupt_kind) = interrupt {
                    interrupt_line.ack(interrupt_kind);
                    registers.set_pc(match interrupt_kind {
                        InterruptKind::VBlank => 0x40,
                        InterruptKind::Stat => 0x48,
                        InterruptKind::Timer => 0x50,
                        InterruptKind::Serial => 0x58,
                        InterruptKind::Joypad => 0x60,
                    })
                } else {
                    registers.set_pc(0)
                }

                let _ = std::mem::replace(self, Self::Complete);
                self.next(registers, interrupt_line)
            }
            Self::Complete => InterruptDispatchExecutionState::Complete,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuOperation {
    EnableInterrupt,
    EnableInterruptNow,
    DisableInterrupt,
    Halt,
}

pub struct Cpu {
    registers: Registers,
    state: State,
    ime: bool,
    enable_ime: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            state: State::NotStarted,
            ime: true,
            enable_ime: false,
        }
    }

    fn prefetch_next(&mut self, cb_prefixed: bool) -> MemoryOperation {
        self.state = State::WaitingPrefetchRead(cb_prefixed);
        MemoryOperation::Read {
            address: self.registers.pc(),
        }
    }

    pub fn tick(&mut self, bus: &Bus, interrupt_line: &mut impl InterruptLine) -> MemoryOperation {
        match &mut self.state {
            State::NotStarted => self.prefetch_next(false),
            State::WaitingPrefetchRead(cb_prefixed) => {
                if !*cb_prefixed {
                    let interrupt = interrupt_line.highest_priority();
                    if self.ime && interrupt.is_some() {
                        self.state = State::StartInterruptDispatch;
                        return self.tick(bus, interrupt_line);
                    }
                }

                if self.enable_ime {
                    self.enable_ime = false;
                    self.ime = true;
                }

                self.registers.set_pc(self.registers.pc().wrapping_add(1));

                self.state = State::DecodeAndExec(bus.data(), *cb_prefixed);
                self.tick(bus, interrupt_line)
            }
            State::DecodeAndExec(opcode, cb_prefixed) => {
                if !*cb_prefixed && *opcode == 0xCB {
                    return self.prefetch_next(true);
                }

                let instruction = match cb_prefixed {
                    true => instructions::CB_PREFIXED_INSTRUCTIONS_TABLE[bus.data() as usize],
                    false => instructions::INSTRUCTIONS_TABLE[bus.data() as usize],
                };

                self.state =
                    State::ExecutingInstruction(instruction, instruction.create_execution());

                self.tick(bus, interrupt_line)
            }
            State::ExecutingInstruction(_instruction, instruction_execution) => {
                match instruction_execution.next(&mut self.registers, bus.data()) {
                    instructions::InstructionExecutionState::YieldMemoryOperation(memory_op) => {
                        memory_op
                    }
                    instructions::InstructionExecutionState::Complete => self.prefetch_next(false),
                    instructions::InstructionExecutionState::YieldCpuOperation(cpu_op) => {
                        match cpu_op {
                            CpuOperation::EnableInterrupt => {
                                self.enable_ime = true;
                                self.ime = false;
                            }
                            CpuOperation::EnableInterruptNow => {
                                self.enable_ime = false;
                                self.ime = true;
                            }
                            CpuOperation::DisableInterrupt => {
                                self.enable_ime = false;
                                self.ime = false;
                            }
                            CpuOperation::Halt => {
                                let memory_op = self.prefetch_next(false);
                                self.state = State::AfterHalt;
                                return memory_op;
                            }
                        }

                        self.tick(bus, interrupt_line)
                    }
                }
            }
            State::StartInterruptDispatch => {
                self.ime = false;
                self.state = State::InterruptDispatching(InterruptDispatchState::Start);
                self.tick(bus, interrupt_line)
            }
            State::InterruptDispatching(interrupt_dispatch_state) => {
                match interrupt_dispatch_state.next(&mut self.registers, interrupt_line) {
                    InterruptDispatchExecutionState::Yield(memory_op) => memory_op,
                    InterruptDispatchExecutionState::Complete => self.prefetch_next(false),
                }
            }
            State::AfterHalt => {
                if interrupt_line.highest_priority().is_some() {
                    if self.ime {
                        self.state = State::StartInterruptDispatch;
                        self.tick(bus, interrupt_line)
                    } else {
                        self.state = State::DecodeAndExec(bus.data(), false);
                        self.tick(bus, interrupt_line)
                    }
                } else {
                    self.state = State::Halted;
                    self.tick(bus, interrupt_line)
                }
            }
            State::Halted => {
                if interrupt_line.flags_not_empty() {
                    self.prefetch_next(false)
                } else {
                    MemoryOperation::None
                }
            }
        }
    }
}
