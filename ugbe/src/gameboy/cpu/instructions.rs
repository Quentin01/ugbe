use std::{borrow::Cow, fmt::Display};

#[derive(Debug, Copy, Clone)]
pub enum AddressBusSource {
    R16(super::registers::R16),
    DecrementR16(super::registers::R16),
    IncrementR16(super::registers::R16),
    High(super::In8),
}

impl AddressBusSource {
    pub fn read_word(&self, cpu_context: &mut super::CpuContext) -> u16 {
        match self {
            Self::R16(reg) => cpu_context.registers.read_word(*reg),
            Self::DecrementR16(reg) => {
                let address = cpu_context.registers.read_word(*reg);
                cpu_context
                    .registers
                    .write_word(*reg, address.wrapping_sub(1));
                address
            }
            Self::IncrementR16(reg) => {
                let address = cpu_context.registers.read_word(*reg);
                cpu_context
                    .registers
                    .write_word(*reg, address.wrapping_add(1));
                address
            }
            Self::High(value) => {
                let value = value.read_byte(cpu_context);
                0xFF00 | value as u16
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryOperation {
    None,
    ChangeAddress(AddressBusSource),
    Read(AddressBusSource),
    Write(AddressBusSource, super::In8),
    CBPrefix,
    PrefetchNext,
}

#[derive(Debug, Copy, Clone)]
pub enum ExecuteOperation {
    None,
    Store8 {
        dst: super::Out8,
        src: super::In8,
    },
    Store16 {
        dst: super::Out16,
        src: super::In16,
    },
    Alu8 {
        dst: super::Out8,
        operation: super::alu::Operation8,
    },
    Alu16 {
        dst: super::Out16,
        operation: super::alu::Operation16,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct MachineCycle {
    // TODO: Should we really expose this publicly?
    pub execute_operation: ExecuteOperation,
    // TODO: Should we really expose this publicly?
    pub memory_operation: MemoryOperation,
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

impl Condition {
    pub fn check(&self, cpu_context: &super::CpuContext) -> bool {
        match self {
            Condition::NZ => !cpu_context.registers.read_flag(super::registers::Flag::Z),
            Condition::Z => cpu_context.registers.read_flag(super::registers::Flag::Z),
            Condition::NC => !cpu_context.registers.read_flag(super::registers::Flag::C),
            Condition::C => cpu_context.registers.read_flag(super::registers::Flag::C),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MachineCycleOperations {
    Conditional {
        condition: Condition,
        ok: &'static [MachineCycle],
        not_ok: &'static [MachineCycle],
    },
    NotConditional(&'static [MachineCycle]),
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    desc: &'static str,
    concrete_desc: Option<&'static str>,
    // TODO: Should we really expose this publicly?
    pub machine_cycles_operations: MachineCycleOperations,
}

impl Instruction {
    pub fn decode(opcode: u8) -> Option<&'static Instruction> {
        INSTRUCTIONS_TABLE[opcode as usize].as_ref()
    }

    // TODO: Remove the option as this can't fail
    pub fn decode_cb_prefixed(opcode: u8) -> Option<&'static Instruction> {
        CB_PREFIXED_INSTRUCTIONS_TABLE[opcode as usize].as_ref()
    }

    pub fn concrete_desc(&self, pc: u16, hardware: &super::Hardware) -> Cow<'static, str> {
        match self.concrete_desc {
            Some(concrete_desc) => {
                if concrete_desc.contains("{u8}") {
                    str::replace(
                        concrete_desc,
                        "{u8}",
                        &format!("${:02x}", hardware.read_byte(pc.wrapping_add(1))),
                    )
                    .into()
                } else if concrete_desc.contains("{u16}") {
                    str::replace(
                        concrete_desc,
                        "{u16}",
                        &format!("${:04x}", hardware.read_word(pc.wrapping_add(1))),
                    )
                    .into()
                } else if concrete_desc.contains("{i8}") {
                    let offset = hardware.read_byte(pc.wrapping_add(1));
                    let dst_pc = (pc.wrapping_add(2)) as i32 + ((offset as i8) as i32);

                    str::replace(
                        concrete_desc,
                        "{i8}",
                        // TODO: Display as a signed hexadecimal integer
                        &format!(
                            "${:02x} (=> ${:04x})",
                            hardware.read_byte(pc.wrapping_add(1)),
                            dst_pc
                        ),
                    )
                    .into()
                } else {
                    concrete_desc.into()
                }
            }
            None => self.desc.into(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.desc)
    }
}

/// Generate a NOP instruction
/// M1:
///     Execute: None
///     Memory: prefetch next instruction
macro_rules! nop_instruction {
    () => {
        Instruction {
            desc: "NOP",
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::None,
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a LD r8, r8 instruction
/// M1:
///     Execute: r8 -> r8
///     Memory: prefetch next instruction
macro_rules! ld_r8_r8_instruction {
    ($dst:ident, $src:ident) => {
        Instruction {
            desc: concat!("LD ", stringify!($dst), ", ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Store8 {
                    dst: super::Out8::R8(super::registers::R8::$dst),
                    src: super::In8::R8(super::registers::R8::$src),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a LD r8, u8 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: data bus -> r8
///     Memory: prefetch next instruction
macro_rules! ld_r8_u8_instruction {
    ($dst:ident) => {
        Instruction {
            desc: concat!("LD ", stringify!($dst), ", u8",),
            concrete_desc: Some(concat!("LD ", stringify!($dst), ", {u8}",)),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$dst),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD r8, (r16) instruction
/// M1:
///     Execute: N/A
///     Memory: Read(r16)
/// M2:
///     Execute: data bus -> r8
///     Memory: prefetch next op
macro_rules! ld_r8_a16_instruction {
    ($dst:ident, $src:ident) => {
        Instruction {
            desc: concat!("LD ", stringify!($src), ", (", stringify!($dst), ")"),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::R16(
                        super::registers::R16::$src,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$dst),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD (r16), r8 instruction
/// M1:
///     Execute: N/A
///     Memory: Write(r16, r8)
/// M2:
///     Execute: N/A
///     Memory: prefetch next op
macro_rules! ld_ar16_r8_instruction {
    ($dst:ident, $src:ident) => {
        Instruction {
            desc: concat!("LD (", stringify!($dst), "), ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::R16(super::registers::R16::$dst),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD (u16), r8 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: data bus -> Y
///     Memory: Read(PC++)
/// M3:
///     Execute: data bus -> X
///     Memory: Write(XY, r8)
/// M4:
///     Execute: N/A
///     Memory: prefetch next op
macro_rules! ld_au16_r8_instruction {
    ($src:ident) => {
        Instruction {
            desc: concat!("LD (u16), ", stringify!($src)),
            concrete_desc: Some(concat!("LD ({u16}), ", stringify!($src))),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::Y),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::X),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::R16(super::registers::R16::XY),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD A, ($FF00+r8) instruction
/// M1:
///     Execute: N/A
///     Memory: ReadHigh(r8)
/// M2:
///     Execute: data bus -> r8
///     Memory: prefetch next instruction
macro_rules! ldh_src_r8_instruction {
    ($src:ident) => {
        Instruction {
            desc: concat!("LD A, (FF00+", stringify!($src), ")"),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::High(
                        super::In8::R8(super::registers::R8::$src),
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::A),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD ($FF00+r8), A instruction
/// M1:
///     Execute: N/A
///     Memory: WriteHigh(r8, A)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! ldh_dst_r8_instruction {
    ($dst:ident) => {
        Instruction {
            desc: concat!("LD (FF00+", stringify!($dst), "), A"),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::High(super::In8::R8(super::registers::R8::$dst)),
                        super::In8::R8(super::registers::R8::A),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD A, ($FF00+u8) instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: N/A
///     Memory: ReadHigh(data bus)
/// M3:
///     Execute: data bus -> A
///     Memory: prefetch next op
macro_rules! ldh_src_u8_instruction {
    () => {
        Instruction {
            desc: concat!("LD A, (FF00+u8)"),
            concrete_desc: Some(concat!("LD A, (FF00+{u8})")),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::A),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::High(
                        super::In8::DataBus,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::A),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD ($FF00+u8), A instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: data bus -> Y
///     Memory: WriteHigh(Y, A)
/// M3:
///     Execute: N/A
///     Memory: prefetch next op
macro_rules! ldh_dst_u8_instruction {
    () => {
        Instruction {
            desc: concat!("LD (FF00+u8), A"),
            concrete_desc: Some(concat!("LD (FF00+{u8}), A")),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::Y),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::High(super::In8::R8(super::registers::R8::Y)),
                        super::In8::R8(super::registers::R8::A),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD r16, u16 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: data bus -> lsb(r16)
///     Memory: Read(PC++)
/// M3:
///     Execute: data bus -> msb(r16)
///     Memory: Read(PC++)
macro_rules! ld_r16_u16_instruction {
    ($dst:ident) => {
        Instruction {
            desc: concat!("LD ", stringify!($dst), ", u16",),
            concrete_desc: Some(concat!("LD ", stringify!($dst), ", {u16}",)),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$dst(
                            super::registers::R16ToR8::Low,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$dst(
                            super::registers::R16ToR8::High,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD (HL++), r8 instruction
/// M1:
///     Execute: N/A
///     Memory: Write(HL++, r8)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! ld_inc_a16_r8_instruction {
    ($src:ident) => {
        Instruction {
            desc: concat!("LD (HL++), ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::IncrementR16(super::registers::R16::HL),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a LD (HL--), r8 instruction
/// M1:
///     Execute: N/A
///     Memory: Write(HL--, r8)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! ld_dec_a16_r8_instruction {
    ($src:ident) => {
        Instruction {
            desc: concat!("LD (HL--), ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::DecrementR16(super::registers::R16::HL),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a INC r8 instruction
/// M1:
///     Execute: r8 + 1 -> r8
///     Memory: prefetch next instruction
macro_rules! inc_r8_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("INC ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::$reg),
                    operation: super::alu::Operation8::Inc(super::In8::R8(
                        super::registers::R8::$reg,
                    )),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a INC r16 instruction
/// M1:
///     Execute: N/A
///     Memory: ChangeAddr(r16++)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! inc_r16_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("INC ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::ChangeAddress(
                        AddressBusSource::IncrementR16(super::registers::R16::$reg),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a DEC r8 instruction
/// M1:
///     Execute: r8 - 1 -> r8
///     Memory: prefetch next instruction
macro_rules! dec_r8_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("DEC ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::$reg),
                    operation: super::alu::Operation8::Dec(super::In8::R8(
                        super::registers::R8::$reg,
                    )),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a DEC r16 instruction
/// M1:
///     Execute: N/A
///     Memory: ChangeAddr(r16--)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! dec_r16_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("DEC ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::ChangeAddress(
                        AddressBusSource::DecrementR16(super::registers::R16::$reg),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a PUSH r16 instruction
/// M1:
///     Execute: N/A
///     Memory: ChangeAddr(SP--)
/// M2:
///     Execute: N/A
///     Memory: Write(SP--, msb(r16))
/// M3:
///     Execute: N/A
///     Memory: Write(SP, lsb(r16))
/// M4:
///     Execute: N/A
///     Memory: prefetch next op
macro_rules! push_r16_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("PUSH ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::ChangeAddress(
                        AddressBusSource::DecrementR16(super::registers::R16::SP),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::DecrementR16(super::registers::R16::SP),
                        super::In8::R8(super::registers::R8::$reg(super::registers::R16ToR8::High)),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::R16(super::registers::R16::SP),
                        super::In8::R8(super::registers::R8::$reg(super::registers::R16ToR8::Low)),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a POP r16 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(SP++)
/// M2:
///     Execute: data bus -> lsb(r16)
///     Memory: Read(SP++)
/// M3:
///     Execute: data bus -> msb(r16)
///     Memory: prefetch next op
macro_rules! pop_r16_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("PUSH ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::SP,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$reg(
                            super::registers::R16ToR8::Low,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::SP,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::$reg(
                            super::registers::R16ToR8::High,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a ADD A, (HL) instruction
/// M1:
///     Execute: N/A
///     Memory: Read(HL)
/// M2:
///     Execute: A + data bus -> A
///     Memory: prefetch next op
macro_rules! add_hl_instruction {
    () => {
        Instruction {
            desc: "ADD A, (HL)",
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::R16(
                        super::registers::R16::HL,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Alu8 {
                        dst: super::Out8::R8(super::registers::R8::A),
                        operation: super::alu::Operation8::Add(
                            super::In8::R8(super::registers::R8::A),
                            super::In8::DataBus,
                        ),
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a SUB r8, r8 instruction
/// M1:
///     Execute: r8 ^ r8 -> r8
///     Memory: prefetch next instruction
macro_rules! sub_r8_r8_instruction {
    ($dst:ident, $src:ident) => {
        Instruction {
            desc: concat!("SUB ", stringify!($dst), ", ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::$dst),
                    operation: super::alu::Operation8::Sub(
                        super::In8::R8(super::registers::R8::$dst),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a XOR r8, r8 instruction
/// M1:
///     Execute: r8 ^ r8 -> r8
///     Memory: prefetch next instruction
macro_rules! xor_r8_r8_instruction {
    ($dst:ident, $src:ident) => {
        Instruction {
            desc: concat!("XOR ", stringify!($dst), ", ", stringify!($src)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::$dst),
                    operation: super::alu::Operation8::Xor(
                        super::In8::R8(super::registers::R8::$dst),
                        super::In8::R8(super::registers::R8::$src),
                    ),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a CP A, (HL) instruction
/// M1:
///     Execute: N/A
///     Memory: Read(HL)
/// M2:
///     Execute: CP(A, data bus)
///     Memory: prefetch next op
macro_rules! cp_hl_instruction {
    () => {
        Instruction {
            desc: "CP A, (HL)",
            concrete_desc: Some("CP A, (HL)"),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::R16(
                        super::registers::R16::HL,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Alu8 {
                        dst: super::Out8::None,
                        operation: super::alu::Operation8::Cp(
                            super::In8::R8(super::registers::R8::A),
                            super::In8::DataBus,
                        ),
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a CP A, u8 instruction
/// M1:
///     Execute: CP(A, u8)
///     Memory: prefetch next instruction
macro_rules! cp_u8_instruction {
    () => {
        Instruction {
            desc: "CP A, u8",
            concrete_desc: Some("CP A, {u8}"),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Alu8 {
                        dst: super::Out8::None,
                        operation: super::alu::Operation8::Cp(
                            super::In8::R8(super::registers::R8::A),
                            super::In8::DataBus,
                        ),
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a RLA instruction
/// M1:
///     Execute: RLA() -> A
///     Memory: prefetch next instruction
macro_rules! rla_instruction {
    () => {
        Instruction {
            desc: concat!("RLA"),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::A),
                    operation: super::alu::Operation8::RlA,
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a JR i8 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: PC + signed data bus -> XY
///     Memory: N/A
/// M3:
///     Execute: XY -> PC
///     Memory: prefetch next instruction
macro_rules! jr_i8_instruction {
    () => {
        Instruction {
            desc: "JR i8",
            concrete_desc: Some("JR {i8}"),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Alu16 {
                        dst: super::Out16::R16(super::registers::R16::XY),
                        operation: super::alu::Operation16::AddWithI8(
                            super::In16::R16(super::registers::R16::PC),
                            super::In8::DataBus,
                        ),
                    },
                    memory_operation: MemoryOperation::None,
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store16 {
                        dst: super::Out16::R16(super::registers::R16::PC),
                        src: super::In16::R16(super::registers::R16::XY),
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a JR CC, i8 instruction
/// Condition:
///     True:
///         M1:
///             Execute: N/A
///             Memory: Read(PC++)
///         M2:
///             Execute: PC + signed data bus -> XY
///             Memory: N/A
///         M3:
///             Execute: XY -> PC
///             Memory: N/A
///         M4:
///             Execute: N/A
///             Memory: prefetch next instruction
///     False:
///         M1:
///             Execute: N/A
///             Memory: Read(PC++)
///         M2:
///             Execute: PC + signed data bus -> XY
///             Memory: N/A
///         M3:
///             Execute: N/A
///             Memory: prefetch next instruction
macro_rules! jr_cc_i8_instruction {
    ($cond:ident) => {
        Instruction {
            desc: concat!("JR ", stringify!($cond), ", i8"),
            concrete_desc: Some(concat!("JR ", stringify!($cond), ", {i8}")),
            machine_cycles_operations: MachineCycleOperations::Conditional {
                condition: Condition::$cond,
                ok: &[
                    MachineCycle {
                        execute_operation: ExecuteOperation::None,
                        memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                            super::registers::R16::PC,
                        )),
                    },
                    MachineCycle {
                        execute_operation: ExecuteOperation::Alu16 {
                            dst: super::Out16::R16(super::registers::R16::XY),
                            operation: super::alu::Operation16::AddWithI8(
                                super::In16::R16(super::registers::R16::PC),
                                super::In8::DataBus,
                            ),
                        },
                        memory_operation: MemoryOperation::None,
                    },
                    MachineCycle {
                        execute_operation: ExecuteOperation::Store16 {
                            dst: super::Out16::R16(super::registers::R16::PC),
                            src: super::In16::R16(super::registers::R16::XY),
                        },
                        memory_operation: MemoryOperation::None,
                    },
                    MachineCycle {
                        execute_operation: ExecuteOperation::None,
                        memory_operation: MemoryOperation::PrefetchNext,
                    },
                ],
                not_ok: &[
                    MachineCycle {
                        execute_operation: ExecuteOperation::None,
                        memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                            super::registers::R16::PC,
                        )),
                    },
                    MachineCycle {
                        execute_operation: ExecuteOperation::Alu16 {
                            dst: super::Out16::R16(super::registers::R16::XY),
                            operation: super::alu::Operation16::AddWithI8(
                                super::In16::R16(super::registers::R16::PC),
                                super::In8::DataBus,
                            ),
                        },
                        memory_operation: MemoryOperation::None,
                    },
                    MachineCycle {
                        execute_operation: ExecuteOperation::None,
                        memory_operation: MemoryOperation::PrefetchNext,
                    },
                ],
            },
        }
    };
}

/// Generate a CALL u16 instruction
/// M1:
///     Execute: N/A
///     Memory: Read(PC++)
/// M2:
///     Execute: data bus -> Y
///     Memory: Read(PC++)
/// M3:
///     Execute: data bus -> X
///     Memory: ChangeAddr(SP--)
/// M4:
///     Execute: N/A
///     Memory: Write(SP--, msb(PC))
/// M5:
///     Execute: N/A
///     Memory: Write(SP, lsb(PC))
/// M6:
///     Execute: XY -> PC
///     Memory: prefetch next op
macro_rules! call_u16_instruction {
    () => {
        Instruction {
            desc: "CALL u16",
            concrete_desc: Some("CALL {u16}"),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::Y),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::X),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::ChangeAddress(
                        AddressBusSource::DecrementR16(super::registers::R16::SP),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::DecrementR16(super::registers::R16::SP),
                        super::In8::R8(super::registers::R8::PC(super::registers::R16ToR8::High)),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::R16(super::registers::R16::SP),
                        super::In8::R8(super::registers::R8::PC(super::registers::R16ToR8::Low)),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store16 {
                        dst: super::Out16::R16(super::registers::R16::PC),
                        src: super::In16::R16(super::registers::R16::XY),
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a RET instruction
/// M1:
///     Execute: N/A
///     Memory: Read(SP++)
/// M2:
///     Execute: data bus -> lsb(PC)
///     Memory: Read(SP++)
/// M3:
///     Execute: data bus -> msb(PC)
///     Memory: N/A
/// M4:
///     Execute: N/A
///     Memory: prefetch next op
macro_rules! ret_instruction {
    () => {
        Instruction {
            desc: "RET",
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::SP,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::PC(
                            super::registers::R16ToR8::Low,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        super::registers::R16::SP,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::Store8 {
                        dst: super::Out8::R8(super::registers::R8::PC(
                            super::registers::R16ToR8::High,
                        )),
                        src: super::In8::DataBus,
                    },
                    memory_operation: MemoryOperation::None,
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        }
    };
}

/// Generate a RL r8 instruction
/// M1:
///     Execute: RL(r8) -> r8
///     Memory: prefetch next instruction
macro_rules! rl_r8_instruction {
    ($reg:ident) => {
        Instruction {
            desc: concat!("RL ", stringify!($reg)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::R8(super::registers::R8::$reg),
                    operation: super::alu::Operation8::Rl(super::In8::R8(
                        super::registers::R8::$reg,
                    )),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a BIT n, r8 instruction
/// M1:
///     Execute: BIT(n, r8)
///     Memory: prefetch next instruction
macro_rules! bit_n_r8_instruction {
    ($dst:ident, $bit:expr) => {
        Instruction {
            desc: concat!("BIT ", stringify!($bit), ", ", stringify!($dst)),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::Alu8 {
                    dst: super::Out8::None,
                    operation: super::alu::Operation8::Bit(
                        $bit,
                        super::In8::R8(super::registers::R8::$dst),
                    ),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        }
    };
}

/// Generate a CB prefix
/// M1:
///     Execute: N/A
///     Memory: Prefetch the CB prefixed instruction
macro_rules! cb_prefix {
    () => {
        Instruction {
            desc: concat!("CB"),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::None,
                memory_operation: MemoryOperation::CBPrefix,
            }]),
        }
    };
}

const INSTRUCTIONS_TABLE: [Option<Instruction>; 0x100] = [
    /* 0x00 */ Some(nop_instruction!()),
    /* 0x01 */ Some(ld_r16_u16_instruction!(BC)),
    /* 0x02 */ Some(ld_ar16_r8_instruction!(BC, A)),
    /* 0x03 */ Some(inc_r16_instruction!(BC)),
    /* 0x04 */ Some(inc_r8_instruction!(B)),
    /* 0x05 */ Some(dec_r8_instruction!(B)),
    /* 0x06 */ Some(ld_r8_u8_instruction!(B)),
    /* 0x07 */ None,
    /* 0x08 */ None,
    /* 0x09 */ None,
    /* 0x0a */ Some(ld_r8_a16_instruction!(A, BC)),
    /* 0x0b */ Some(dec_r16_instruction!(BC)),
    /* 0x0c */ Some(inc_r8_instruction!(C)),
    /* 0x0d */ Some(dec_r8_instruction!(C)),
    /* 0x0e */ Some(ld_r8_u8_instruction!(C)),
    /* 0x0f */ None,
    /* 0x10 */ None,
    /* 0x11 */ Some(ld_r16_u16_instruction!(DE)),
    /* 0x12 */ Some(ld_ar16_r8_instruction!(DE, A)),
    /* 0x13 */ Some(inc_r16_instruction!(DE)),
    /* 0x14 */ Some(inc_r8_instruction!(D)),
    /* 0x15 */ Some(dec_r8_instruction!(D)),
    /* 0x16 */ Some(ld_r8_u8_instruction!(D)),
    /* 0x17 */ Some(rla_instruction!()),
    /* 0x18 */ Some(jr_i8_instruction!()),
    /* 0x19 */ None,
    /* 0x1a */ Some(ld_r8_a16_instruction!(A, DE)),
    /* 0x1b */ Some(dec_r16_instruction!(DE)),
    /* 0x1c */ Some(inc_r8_instruction!(E)),
    /* 0x1d */ Some(dec_r8_instruction!(E)),
    /* 0x1e */ Some(ld_r8_u8_instruction!(E)),
    /* 0x1f */ None,
    /* 0x20 */ Some(jr_cc_i8_instruction!(NZ)),
    /* 0x21 */ Some(ld_r16_u16_instruction!(HL)),
    /* 0x22 */ Some(ld_inc_a16_r8_instruction!(A)),
    /* 0x23 */ Some(inc_r16_instruction!(HL)),
    /* 0x24 */ Some(inc_r8_instruction!(H)),
    /* 0x25 */ Some(dec_r8_instruction!(H)),
    /* 0x26 */ Some(ld_r8_u8_instruction!(H)),
    /* 0x27 */ None,
    /* 0x28 */ Some(jr_cc_i8_instruction!(Z)),
    /* 0x29 */ None,
    /* 0x2a */ None,
    /* 0x2b */ Some(dec_r16_instruction!(HL)),
    /* 0x2c */ Some(inc_r8_instruction!(L)),
    /* 0x2d */ Some(dec_r8_instruction!(L)),
    /* 0x2e */ Some(ld_r8_u8_instruction!(L)),
    /* 0x2f */ None,
    /* 0x30 */ Some(jr_cc_i8_instruction!(NC)),
    /* 0x31 */ Some(ld_r16_u16_instruction!(SP)),
    /* 0x32 */ Some(ld_dec_a16_r8_instruction!(A)),
    /* 0x33 */ Some(inc_r16_instruction!(SP)),
    /* 0x34 */ None,
    /* 0x35 */ None,
    /* 0x36 */ None,
    /* 0x37 */ None,
    /* 0x38 */ Some(jr_cc_i8_instruction!(C)),
    /* 0x39 */ None,
    /* 0x3a */ None,
    /* 0x3b */ Some(dec_r16_instruction!(SP)),
    /* 0x3c */ Some(inc_r8_instruction!(A)),
    /* 0x3d */ Some(dec_r8_instruction!(A)),
    /* 0x3e */ Some(ld_r8_u8_instruction!(A)),
    /* 0x3f */ None,
    /* 0x40 */ Some(ld_r8_r8_instruction!(B, B)),
    /* 0x41 */ Some(ld_r8_r8_instruction!(B, C)),
    /* 0x42 */ Some(ld_r8_r8_instruction!(B, D)),
    /* 0x43 */ Some(ld_r8_r8_instruction!(B, E)),
    /* 0x44 */ Some(ld_r8_r8_instruction!(B, H)),
    /* 0x45 */ Some(ld_r8_r8_instruction!(B, L)),
    /* 0x46 */ Some(ld_r8_a16_instruction!(B, HL)),
    /* 0x47 */ Some(ld_r8_r8_instruction!(B, A)),
    /* 0x48 */ Some(ld_r8_r8_instruction!(C, B)),
    /* 0x49 */ Some(ld_r8_r8_instruction!(C, C)),
    /* 0x4a */ Some(ld_r8_r8_instruction!(C, D)),
    /* 0x4b */ Some(ld_r8_r8_instruction!(C, E)),
    /* 0x4c */ Some(ld_r8_r8_instruction!(C, H)),
    /* 0x4d */ Some(ld_r8_r8_instruction!(C, L)),
    /* 0x4e */ Some(ld_r8_a16_instruction!(C, HL)),
    /* 0x4f */ Some(ld_r8_r8_instruction!(C, A)),
    /* 0x50 */ Some(ld_r8_r8_instruction!(D, B)),
    /* 0x51 */ Some(ld_r8_r8_instruction!(D, C)),
    /* 0x52 */ Some(ld_r8_r8_instruction!(D, D)),
    /* 0x53 */ Some(ld_r8_r8_instruction!(D, E)),
    /* 0x54 */ Some(ld_r8_r8_instruction!(D, H)),
    /* 0x55 */ Some(ld_r8_r8_instruction!(D, L)),
    /* 0x56 */ Some(ld_r8_a16_instruction!(D, HL)),
    /* 0x57 */ Some(ld_r8_r8_instruction!(D, A)),
    /* 0x58 */ Some(ld_r8_r8_instruction!(E, B)),
    /* 0x59 */ Some(ld_r8_r8_instruction!(E, C)),
    /* 0x5a */ Some(ld_r8_r8_instruction!(E, D)),
    /* 0x5b */ Some(ld_r8_r8_instruction!(E, E)),
    /* 0x5c */ Some(ld_r8_r8_instruction!(E, H)),
    /* 0x5d */ Some(ld_r8_r8_instruction!(E, L)),
    /* 0x5e */ Some(ld_r8_a16_instruction!(E, HL)),
    /* 0x5f */ Some(ld_r8_r8_instruction!(E, A)),
    /* 0x60 */ Some(ld_r8_r8_instruction!(H, B)),
    /* 0x61 */ Some(ld_r8_r8_instruction!(H, C)),
    /* 0x62 */ Some(ld_r8_r8_instruction!(H, D)),
    /* 0x63 */ Some(ld_r8_r8_instruction!(H, E)),
    /* 0x64 */ Some(ld_r8_r8_instruction!(H, H)),
    /* 0x65 */ Some(ld_r8_r8_instruction!(H, L)),
    /* 0x66 */ Some(ld_r8_a16_instruction!(H, HL)),
    /* 0x67 */ Some(ld_r8_r8_instruction!(H, A)),
    /* 0x68 */ Some(ld_r8_r8_instruction!(L, B)),
    /* 0x69 */ Some(ld_r8_r8_instruction!(L, C)),
    /* 0x6a */ Some(ld_r8_r8_instruction!(L, D)),
    /* 0x6b */ Some(ld_r8_r8_instruction!(L, E)),
    /* 0x6c */ Some(ld_r8_r8_instruction!(L, H)),
    /* 0x6d */ Some(ld_r8_r8_instruction!(L, L)),
    /* 0x6e */ Some(ld_r8_a16_instruction!(L, HL)),
    /* 0x6f */ Some(ld_r8_r8_instruction!(L, A)),
    /* 0x70 */ Some(ld_ar16_r8_instruction!(HL, B)),
    /* 0x71 */ Some(ld_ar16_r8_instruction!(HL, C)),
    /* 0x72 */ Some(ld_ar16_r8_instruction!(HL, D)),
    /* 0x73 */ Some(ld_ar16_r8_instruction!(HL, E)),
    /* 0x74 */ Some(ld_ar16_r8_instruction!(HL, H)),
    /* 0x75 */ Some(ld_ar16_r8_instruction!(HL, L)),
    /* 0x76 */ None,
    /* 0x77 */ Some(ld_ar16_r8_instruction!(HL, A)),
    /* 0x78 */ Some(ld_r8_r8_instruction!(A, B)),
    /* 0x79 */ Some(ld_r8_r8_instruction!(A, C)),
    /* 0x7a */ Some(ld_r8_r8_instruction!(A, D)),
    /* 0x7b */ Some(ld_r8_r8_instruction!(A, E)),
    /* 0x7c */ Some(ld_r8_r8_instruction!(A, H)),
    /* 0x7d */ Some(ld_r8_r8_instruction!(A, L)),
    /* 0x7e */ Some(ld_r8_a16_instruction!(A, HL)),
    /* 0x7f */ Some(ld_r8_r8_instruction!(A, A)),
    /* 0x80 */ None,
    /* 0x81 */ None,
    /* 0x82 */ None,
    /* 0x83 */ None,
    /* 0x84 */ None,
    /* 0x85 */ None,
    /* 0x86 */ Some(add_hl_instruction!()),
    /* 0x87 */ None,
    /* 0x88 */ None,
    /* 0x89 */ None,
    /* 0x8a */ None,
    /* 0x8b */ None,
    /* 0x8c */ None,
    /* 0x8d */ None,
    /* 0x8e */ None,
    /* 0x8f */ None,
    /* 0x90 */ Some(sub_r8_r8_instruction!(A, B)),
    /* 0x91 */ Some(sub_r8_r8_instruction!(A, C)),
    /* 0x92 */ Some(sub_r8_r8_instruction!(A, D)),
    /* 0x93 */ Some(sub_r8_r8_instruction!(A, E)),
    /* 0x94 */ Some(sub_r8_r8_instruction!(A, H)),
    /* 0x95 */ Some(sub_r8_r8_instruction!(A, L)),
    /* 0x96 */ None,
    /* 0x97 */ Some(sub_r8_r8_instruction!(A, A)),
    /* 0x98 */ None,
    /* 0x99 */ None,
    /* 0x9a */ None,
    /* 0x9b */ None,
    /* 0x9c */ None,
    /* 0x9d */ None,
    /* 0x9e */ None,
    /* 0x9f */ None,
    /* 0xa0 */ None,
    /* 0xa1 */ None,
    /* 0xa2 */ None,
    /* 0xa3 */ None,
    /* 0xa4 */ None,
    /* 0xa5 */ None,
    /* 0xa6 */ None,
    /* 0xa7 */ None,
    /* 0xa8 */ Some(xor_r8_r8_instruction!(A, B)),
    /* 0xa9 */ Some(xor_r8_r8_instruction!(A, C)),
    /* 0xaa */ Some(xor_r8_r8_instruction!(A, D)),
    /* 0xab */ Some(xor_r8_r8_instruction!(A, E)),
    /* 0xac */ Some(xor_r8_r8_instruction!(A, H)),
    /* 0xad */ Some(xor_r8_r8_instruction!(A, L)),
    /* 0xae */ None,
    /* 0xaf */ Some(xor_r8_r8_instruction!(A, A)),
    /* 0xb0 */ None,
    /* 0xb1 */ None,
    /* 0xb2 */ None,
    /* 0xb3 */ None,
    /* 0xb4 */ None,
    /* 0xb5 */ None,
    /* 0xb6 */ None,
    /* 0xb7 */ None,
    /* 0xb8 */ None,
    /* 0xb9 */ None,
    /* 0xba */ None,
    /* 0xbb */ None,
    /* 0xbc */ None,
    /* 0xbd */ None,
    /* 0xbe */ Some(cp_hl_instruction!()),
    /* 0xbf */ None,
    /* 0xc0 */ None,
    /* 0xc1 */ Some(pop_r16_instruction!(BC)),
    /* 0xc2 */ None,
    /* 0xc3 */ None,
    /* 0xc4 */ None,
    /* 0xc5 */ Some(push_r16_instruction!(BC)),
    /* 0xc6 */ None,
    /* 0xc7 */ None,
    /* 0xc8 */ None,
    /* 0xc9 */ Some(ret_instruction!()),
    /* 0xca */ None,
    /* 0xcb */ Some(cb_prefix!()),
    /* 0xcc */ None,
    /* 0xcd */ Some(call_u16_instruction!()),
    /* 0xce */ None,
    /* 0xcf */ None,
    /* 0xd0 */ None,
    /* 0xd1 */ Some(pop_r16_instruction!(DE)),
    /* 0xd2 */ None,
    /* 0xd3 */ None,
    /* 0xd4 */ None,
    /* 0xd5 */ Some(push_r16_instruction!(DE)),
    /* 0xd6 */ None,
    /* 0xd7 */ None,
    /* 0xd8 */ None,
    /* 0xd9 */ None,
    /* 0xda */ None,
    /* 0xdb */ None,
    /* 0xdc */ None,
    /* 0xdd */ None,
    /* 0xde */ None,
    /* 0xdf */ None,
    /* 0xe0 */ Some(ldh_dst_u8_instruction!()),
    /* 0xe1 */ Some(pop_r16_instruction!(HL)),
    /* 0xe2 */ Some(ldh_dst_r8_instruction!(C)),
    /* 0xe3 */ None,
    /* 0xe4 */ None,
    /* 0xe5 */ Some(push_r16_instruction!(HL)),
    /* 0xe6 */ None,
    /* 0xe7 */ None,
    /* 0xe8 */ None,
    /* 0xe9 */ None,
    /* 0xea */ Some(ld_au16_r8_instruction!(A)),
    /* 0xeb */ None,
    /* 0xec */ None,
    /* 0xed */ None,
    /* 0xee */ None,
    /* 0xef */ None,
    /* 0xf0 */ Some(ldh_src_u8_instruction!()),
    /* 0xf1 */ Some(pop_r16_instruction!(AF)),
    /* 0xf2 */ Some(ldh_src_r8_instruction!(C)),
    /* 0xf3 */ None,
    /* 0xf4 */ None,
    /* 0xf5 */ Some(push_r16_instruction!(AF)),
    /* 0xf6 */ None,
    /* 0xf7 */ None,
    /* 0xf8 */ None,
    /* 0xf9 */ None,
    /* 0xfa */ None,
    /* 0xfb */ None,
    /* 0xfc */ None,
    /* 0xfd */ None,
    /* 0xfe */ Some(cp_u8_instruction!()),
    /* 0xff */ None,
];

// TODO: Remove the option as this can't fail
const CB_PREFIXED_INSTRUCTIONS_TABLE: [Option<Instruction>; 0x100] = [
    /* 0x00 */ None,
    /* 0x01 */ None,
    /* 0x02 */ None,
    /* 0x03 */ None,
    /* 0x04 */ None,
    /* 0x05 */ None,
    /* 0x06 */ None,
    /* 0x07 */ None,
    /* 0x08 */ None,
    /* 0x09 */ None,
    /* 0x0a */ None,
    /* 0x0b */ None,
    /* 0x0c */ None,
    /* 0x0d */ None,
    /* 0x0e */ None,
    /* 0x0f */ None,
    /* 0x10 */ Some(rl_r8_instruction!(B)),
    /* 0x11 */ Some(rl_r8_instruction!(C)),
    /* 0x12 */ Some(rl_r8_instruction!(D)),
    /* 0x13 */ Some(rl_r8_instruction!(E)),
    /* 0x14 */ Some(rl_r8_instruction!(H)),
    /* 0x15 */ Some(rl_r8_instruction!(L)),
    /* 0x16 */ None,
    /* 0x17 */ Some(rl_r8_instruction!(A)),
    /* 0x18 */ None,
    /* 0x19 */ None,
    /* 0x1a */ None,
    /* 0x1b */ None,
    /* 0x1c */ None,
    /* 0x1d */ None,
    /* 0x1e */ None,
    /* 0x1f */ None,
    /* 0x20 */ None,
    /* 0x21 */ None,
    /* 0x22 */ None,
    /* 0x23 */ None,
    /* 0x24 */ None,
    /* 0x25 */ None,
    /* 0x26 */ None,
    /* 0x27 */ None,
    /* 0x28 */ None,
    /* 0x29 */ None,
    /* 0x2a */ None,
    /* 0x2b */ None,
    /* 0x2c */ None,
    /* 0x2d */ None,
    /* 0x2e */ None,
    /* 0x2f */ None,
    /* 0x30 */ None,
    /* 0x31 */ None,
    /* 0x32 */ None,
    /* 0x33 */ None,
    /* 0x34 */ None,
    /* 0x35 */ None,
    /* 0x36 */ None,
    /* 0x37 */ None,
    /* 0x38 */ None,
    /* 0x39 */ None,
    /* 0x3a */ None,
    /* 0x3b */ None,
    /* 0x3c */ None,
    /* 0x3d */ None,
    /* 0x3e */ None,
    /* 0x3f */ None,
    /* 0x40 */ Some(bit_n_r8_instruction!(B, 0)),
    /* 0x41 */ Some(bit_n_r8_instruction!(C, 0)),
    /* 0x42 */ Some(bit_n_r8_instruction!(D, 0)),
    /* 0x43 */ Some(bit_n_r8_instruction!(E, 0)),
    /* 0x44 */ Some(bit_n_r8_instruction!(H, 0)),
    /* 0x45 */ Some(bit_n_r8_instruction!(L, 0)),
    /* 0x46 */ None,
    /* 0x47 */ Some(bit_n_r8_instruction!(A, 0)),
    /* 0x48 */ Some(bit_n_r8_instruction!(B, 1)),
    /* 0x49 */ Some(bit_n_r8_instruction!(C, 1)),
    /* 0x4a */ Some(bit_n_r8_instruction!(D, 1)),
    /* 0x4b */ Some(bit_n_r8_instruction!(E, 1)),
    /* 0x4c */ Some(bit_n_r8_instruction!(H, 1)),
    /* 0x4d */ Some(bit_n_r8_instruction!(L, 1)),
    /* 0x4e */ None,
    /* 0x4f */ Some(bit_n_r8_instruction!(A, 1)),
    /* 0x50 */ Some(bit_n_r8_instruction!(B, 2)),
    /* 0x51 */ Some(bit_n_r8_instruction!(C, 2)),
    /* 0x52 */ Some(bit_n_r8_instruction!(D, 2)),
    /* 0x53 */ Some(bit_n_r8_instruction!(E, 2)),
    /* 0x54 */ Some(bit_n_r8_instruction!(H, 2)),
    /* 0x55 */ Some(bit_n_r8_instruction!(L, 2)),
    /* 0x56 */ None,
    /* 0x57 */ Some(bit_n_r8_instruction!(A, 2)),
    /* 0x58 */ Some(bit_n_r8_instruction!(B, 3)),
    /* 0x59 */ Some(bit_n_r8_instruction!(C, 3)),
    /* 0x5a */ Some(bit_n_r8_instruction!(D, 3)),
    /* 0x5b */ Some(bit_n_r8_instruction!(E, 3)),
    /* 0x5c */ Some(bit_n_r8_instruction!(H, 3)),
    /* 0x5d */ Some(bit_n_r8_instruction!(L, 3)),
    /* 0x5e */ None,
    /* 0x5f */ Some(bit_n_r8_instruction!(A, 3)),
    /* 0x60 */ Some(bit_n_r8_instruction!(B, 4)),
    /* 0x61 */ Some(bit_n_r8_instruction!(C, 4)),
    /* 0x62 */ Some(bit_n_r8_instruction!(D, 4)),
    /* 0x63 */ Some(bit_n_r8_instruction!(E, 4)),
    /* 0x64 */ Some(bit_n_r8_instruction!(H, 4)),
    /* 0x65 */ Some(bit_n_r8_instruction!(L, 4)),
    /* 0x66 */ None,
    /* 0x67 */ Some(bit_n_r8_instruction!(A, 4)),
    /* 0x68 */ Some(bit_n_r8_instruction!(B, 5)),
    /* 0x69 */ Some(bit_n_r8_instruction!(C, 5)),
    /* 0x6a */ Some(bit_n_r8_instruction!(D, 5)),
    /* 0x6b */ Some(bit_n_r8_instruction!(E, 5)),
    /* 0x6c */ Some(bit_n_r8_instruction!(H, 5)),
    /* 0x6d */ Some(bit_n_r8_instruction!(L, 5)),
    /* 0x6e */ None,
    /* 0x6f */ Some(bit_n_r8_instruction!(A, 5)),
    /* 0x70 */ Some(bit_n_r8_instruction!(B, 6)),
    /* 0x71 */ Some(bit_n_r8_instruction!(C, 6)),
    /* 0x72 */ Some(bit_n_r8_instruction!(D, 6)),
    /* 0x73 */ Some(bit_n_r8_instruction!(E, 6)),
    /* 0x74 */ Some(bit_n_r8_instruction!(H, 6)),
    /* 0x75 */ Some(bit_n_r8_instruction!(L, 6)),
    /* 0x76 */ None,
    /* 0x77 */ Some(bit_n_r8_instruction!(A, 6)),
    /* 0x78 */ Some(bit_n_r8_instruction!(B, 7)),
    /* 0x79 */ Some(bit_n_r8_instruction!(C, 7)),
    /* 0x7a */ Some(bit_n_r8_instruction!(D, 7)),
    /* 0x7b */ Some(bit_n_r8_instruction!(E, 7)),
    /* 0x7c */ Some(bit_n_r8_instruction!(H, 7)),
    /* 0x7d */ Some(bit_n_r8_instruction!(L, 7)),
    /* 0x7e */ None,
    /* 0x7f */ Some(bit_n_r8_instruction!(A, 7)),
    /* 0x80 */ None,
    /* 0x81 */ None,
    /* 0x82 */ None,
    /* 0x83 */ None,
    /* 0x84 */ None,
    /* 0x85 */ None,
    /* 0x86 */ None,
    /* 0x87 */ None,
    /* 0x88 */ None,
    /* 0x89 */ None,
    /* 0x8a */ None,
    /* 0x8b */ None,
    /* 0x8c */ None,
    /* 0x8d */ None,
    /* 0x8e */ None,
    /* 0x8f */ None,
    /* 0x90 */ None,
    /* 0x91 */ None,
    /* 0x92 */ None,
    /* 0x93 */ None,
    /* 0x94 */ None,
    /* 0x95 */ None,
    /* 0x96 */ None,
    /* 0x97 */ None,
    /* 0x98 */ None,
    /* 0x99 */ None,
    /* 0x9a */ None,
    /* 0x9b */ None,
    /* 0x9c */ None,
    /* 0x9d */ None,
    /* 0x9e */ None,
    /* 0x9f */ None,
    /* 0xa0 */ None,
    /* 0xa1 */ None,
    /* 0xa2 */ None,
    /* 0xa3 */ None,
    /* 0xa4 */ None,
    /* 0xa5 */ None,
    /* 0xa6 */ None,
    /* 0xa7 */ None,
    /* 0xa8 */ None,
    /* 0xa9 */ None,
    /* 0xaa */ None,
    /* 0xab */ None,
    /* 0xac */ None,
    /* 0xad */ None,
    /* 0xae */ None,
    /* 0xaf */ None,
    /* 0xb0 */ None,
    /* 0xb1 */ None,
    /* 0xb2 */ None,
    /* 0xb3 */ None,
    /* 0xb4 */ None,
    /* 0xb5 */ None,
    /* 0xb6 */ None,
    /* 0xb7 */ None,
    /* 0xb8 */ None,
    /* 0xb9 */ None,
    /* 0xba */ None,
    /* 0xbb */ None,
    /* 0xbc */ None,
    /* 0xbd */ None,
    /* 0xbe */ None,
    /* 0xbf */ None,
    /* 0xc0 */ None,
    /* 0xc1 */ None,
    /* 0xc2 */ None,
    /* 0xc3 */ None,
    /* 0xc4 */ None,
    /* 0xc5 */ None,
    /* 0xc6 */ None,
    /* 0xc7 */ None,
    /* 0xc8 */ None,
    /* 0xc9 */ None,
    /* 0xca */ None,
    /* 0xcb */ None,
    /* 0xcc */ None,
    /* 0xcd */ None,
    /* 0xce */ None,
    /* 0xcf */ None,
    /* 0xd0 */ None,
    /* 0xd1 */ None,
    /* 0xd2 */ None,
    /* 0xd3 */ None,
    /* 0xd4 */ None,
    /* 0xd5 */ None,
    /* 0xd6 */ None,
    /* 0xd7 */ None,
    /* 0xd8 */ None,
    /* 0xd9 */ None,
    /* 0xda */ None,
    /* 0xdb */ None,
    /* 0xdc */ None,
    /* 0xdd */ None,
    /* 0xde */ None,
    /* 0xdf */ None,
    /* 0xe0 */ None,
    /* 0xe1 */ None,
    /* 0xe2 */ None,
    /* 0xe3 */ None,
    /* 0xe4 */ None,
    /* 0xe5 */ None,
    /* 0xe6 */ None,
    /* 0xe7 */ None,
    /* 0xe8 */ None,
    /* 0xe9 */ None,
    /* 0xea */ None,
    /* 0xeb */ None,
    /* 0xec */ None,
    /* 0xed */ None,
    /* 0xee */ None,
    /* 0xef */ None,
    /* 0xf0 */ None,
    /* 0xf1 */ None,
    /* 0xf2 */ None,
    /* 0xf3 */ None,
    /* 0xf4 */ None,
    /* 0xf5 */ None,
    /* 0xf6 */ None,
    /* 0xf7 */ None,
    /* 0xf8 */ None,
    /* 0xf9 */ None,
    /* 0xfa */ None,
    /* 0xfb */ None,
    /* 0xfc */ None,
    /* 0xfd */ None,
    /* 0xfe */ None,
    /* 0xff */ None,
];
