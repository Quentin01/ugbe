use std::{borrow::Cow, fmt::Display};

use super::registers::{self};

#[derive(Debug, Copy, Clone)]
pub enum AddressBusSource {
    IncrementR16(super::registers::R16),
}

#[derive(Debug, Copy, Clone)]
pub enum DataBusSource {
    R8(super::registers::R8),
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryOperation {
    None,
    Read(AddressBusSource),
    Write(AddressBusSource, DataBusSource),
    CBPrefix,
    PrefetchNext,
}

#[derive(Debug, Copy, Clone)]
pub enum In8 {
    DataBus,
    R8(registers::R8),
    Xor(registers::R8, registers::R8),
}

#[derive(Debug, Copy, Clone)]
pub enum ExecuteOperation {
    None,
    StoreInR8 { dst: registers::R8, src: In8 },
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
    pub fn check(&self, registers: &registers::Registers) -> bool {
        match self {
            Condition::NZ => !registers.read_flag(registers::Flag::Z),
            Condition::Z => registers.read_flag(registers::Flag::Z),
            Condition::NC => !registers.read_flag(registers::Flag::C),
            Condition::C => registers.read_flag(registers::Flag::C),
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
                if concrete_desc.contains("{imm8}") {
                    str::replace(
                        concrete_desc,
                        "{imm8}",
                        &format!("${:02x}", hardware.read_byte(pc)),
                    )
                    .into()
                } else if concrete_desc.contains("{imm16}") {
                    str::replace(
                        concrete_desc,
                        "{imm16}",
                        &format!("${:04x}", hardware.read_word(pc)),
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

macro_rules! invalid_instruction {
    () => {
        None
    };
}

/// Generate a NOP instruction
/// M1:
///     Execute: None
///     Memory: prefetch next instruction
macro_rules! nop_instruction {
    () => {
        Some(Instruction {
            desc: "NOP",
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::None,
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        })
    };
}

/// Generate a LD r8, r8 instruction
/// M1:
///     Execute: r8 -> r8
///     Memory: prefetch next instruction
macro_rules! ld_r8_r8_instruction {
    ($dst:ident, $src:ident) => {
        Some(Instruction {
            desc: concat!("LD ", stringify!($dst), ", ", stringify!($src),),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::StoreInR8 {
                    dst: registers::R8::$dst,
                    src: In8::R8(registers::R8::$dst),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        })
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
        Some(Instruction {
            desc: concat!("LD ", stringify!($dst), ", u16",),
            concrete_desc: Some(concat!("LD ", stringify!($dst), ", {imm16}",)),
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::StoreInR8 {
                        dst: registers::R8::$dst(registers::R16ToR8::Low),
                        src: In8::DataBus,
                    },
                    memory_operation: MemoryOperation::Read(AddressBusSource::IncrementR16(
                        registers::R16::PC,
                    )),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::StoreInR8 {
                        dst: registers::R8::$dst(registers::R16ToR8::High),
                        src: In8::DataBus,
                    },
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        })
    };
}

/// Generate a LD (r16--), r8 instruction
/// M1:
///     Execute: N/A
///     Memory: Write(HL--, r8)
/// M2:
///     Execute: N/A
///     Memory: prefetch next instruction
macro_rules! ld_dec_r16_r8 {
    ($dst:ident, $src:ident) => {
        Some(Instruction {
            desc: concat!("LD (", stringify!($dst), "--), ", stringify!($src),),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::Write(
                        AddressBusSource::IncrementR16(registers::R16::$dst),
                        DataBusSource::R8(registers::R8::$src),
                    ),
                },
                MachineCycle {
                    execute_operation: ExecuteOperation::None,
                    memory_operation: MemoryOperation::PrefetchNext,
                },
            ]),
        })
    };
}

/// Generate a XOR r8, r8 instruction
/// M1:
///     Execute: r8 -> r8 ^ r8
///     Memory: prefetch next instruction
macro_rules! xor_r8_r8_instruction {
    ($dst:ident, $src:ident) => {
        Some(Instruction {
            desc: concat!("XOR ", stringify!($dst), ", ", stringify!($src),),
            concrete_desc: None,
            machine_cycles_operations: MachineCycleOperations::NotConditional(&[MachineCycle {
                execute_operation: ExecuteOperation::StoreInR8 {
                    dst: registers::R8::$dst,
                    src: In8::Xor(registers::R8::$dst, registers::R8::$src),
                },
                memory_operation: MemoryOperation::PrefetchNext,
            }]),
        })
    };
}

const INSTRUCTIONS_TABLE: [Option<Instruction>; 0x100] = [
    /* 0x00 */ nop_instruction!(),
    /* 0x01 */ ld_r16_u16_instruction!(BC),
    /* 0x02 */ invalid_instruction!(),
    /* 0x03 */ invalid_instruction!(),
    /* 0x04 */ invalid_instruction!(),
    /* 0x05 */ invalid_instruction!(),
    /* 0x06 */ invalid_instruction!(),
    /* 0x07 */ invalid_instruction!(),
    /* 0x08 */ invalid_instruction!(),
    /* 0x09 */ invalid_instruction!(),
    /* 0x0a */ invalid_instruction!(),
    /* 0x0b */ invalid_instruction!(),
    /* 0x0c */ invalid_instruction!(),
    /* 0x0d */ invalid_instruction!(),
    /* 0x0e */ invalid_instruction!(),
    /* 0x0f */ invalid_instruction!(),
    /* 0x10 */ invalid_instruction!(),
    /* 0x11 */ ld_r16_u16_instruction!(DE),
    /* 0x12 */ invalid_instruction!(),
    /* 0x13 */ invalid_instruction!(),
    /* 0x14 */ invalid_instruction!(),
    /* 0x15 */ invalid_instruction!(),
    /* 0x16 */ invalid_instruction!(),
    /* 0x17 */ invalid_instruction!(),
    /* 0x18 */ invalid_instruction!(),
    /* 0x19 */ invalid_instruction!(),
    /* 0x1a */ invalid_instruction!(),
    /* 0x1b */ invalid_instruction!(),
    /* 0x1c */ invalid_instruction!(),
    /* 0x1d */ invalid_instruction!(),
    /* 0x1e */ invalid_instruction!(),
    /* 0x1f */ invalid_instruction!(),
    /* 0x20 */ invalid_instruction!(),
    /* 0x21 */ ld_r16_u16_instruction!(HL),
    /* 0x22 */ invalid_instruction!(),
    /* 0x23 */ invalid_instruction!(),
    /* 0x24 */ invalid_instruction!(),
    /* 0x25 */ invalid_instruction!(),
    /* 0x26 */ invalid_instruction!(),
    /* 0x27 */ invalid_instruction!(),
    /* 0x28 */ invalid_instruction!(),
    /* 0x29 */ invalid_instruction!(),
    /* 0x2a */ invalid_instruction!(),
    /* 0x2b */ invalid_instruction!(),
    /* 0x2c */ invalid_instruction!(),
    /* 0x2d */ invalid_instruction!(),
    /* 0x2e */ invalid_instruction!(),
    /* 0x2f */ invalid_instruction!(),
    /* 0x30 */ invalid_instruction!(),
    /* 0x31 */ ld_r16_u16_instruction!(SP),
    /* 0x32 */ ld_dec_r16_r8!(HL, A),
    /* 0x33 */ invalid_instruction!(),
    /* 0x34 */ invalid_instruction!(),
    /* 0x35 */ invalid_instruction!(),
    /* 0x36 */ invalid_instruction!(),
    /* 0x37 */ invalid_instruction!(),
    /* 0x38 */ invalid_instruction!(),
    /* 0x39 */ invalid_instruction!(),
    /* 0x3a */ invalid_instruction!(),
    /* 0x3b */ invalid_instruction!(),
    /* 0x3c */ invalid_instruction!(),
    /* 0x3d */ invalid_instruction!(),
    /* 0x3e */ invalid_instruction!(),
    /* 0x3f */ invalid_instruction!(),
    /* 0x40 */ ld_r8_r8_instruction!(B, B),
    /* 0x41 */ invalid_instruction!(),
    /* 0x42 */ invalid_instruction!(),
    /* 0x43 */ invalid_instruction!(),
    /* 0x44 */ invalid_instruction!(),
    /* 0x45 */ invalid_instruction!(),
    /* 0x46 */ invalid_instruction!(),
    /* 0x47 */ invalid_instruction!(),
    /* 0x48 */ invalid_instruction!(),
    /* 0x49 */ invalid_instruction!(),
    /* 0x4a */ invalid_instruction!(),
    /* 0x4b */ invalid_instruction!(),
    /* 0x4c */ invalid_instruction!(),
    /* 0x4d */ invalid_instruction!(),
    /* 0x4e */ invalid_instruction!(),
    /* 0x4f */ invalid_instruction!(),
    /* 0x50 */ invalid_instruction!(),
    /* 0x51 */ invalid_instruction!(),
    /* 0x52 */ invalid_instruction!(),
    /* 0x53 */ invalid_instruction!(),
    /* 0x54 */ invalid_instruction!(),
    /* 0x55 */ invalid_instruction!(),
    /* 0x56 */ invalid_instruction!(),
    /* 0x57 */ invalid_instruction!(),
    /* 0x58 */ invalid_instruction!(),
    /* 0x59 */ invalid_instruction!(),
    /* 0x5a */ invalid_instruction!(),
    /* 0x5b */ invalid_instruction!(),
    /* 0x5c */ invalid_instruction!(),
    /* 0x5d */ invalid_instruction!(),
    /* 0x5e */ invalid_instruction!(),
    /* 0x5f */ invalid_instruction!(),
    /* 0x60 */ invalid_instruction!(),
    /* 0x61 */ invalid_instruction!(),
    /* 0x62 */ invalid_instruction!(),
    /* 0x63 */ invalid_instruction!(),
    /* 0x64 */ invalid_instruction!(),
    /* 0x65 */ invalid_instruction!(),
    /* 0x66 */ invalid_instruction!(),
    /* 0x67 */ invalid_instruction!(),
    /* 0x68 */ invalid_instruction!(),
    /* 0x69 */ invalid_instruction!(),
    /* 0x6a */ invalid_instruction!(),
    /* 0x6b */ invalid_instruction!(),
    /* 0x6c */ invalid_instruction!(),
    /* 0x6d */ invalid_instruction!(),
    /* 0x6e */ invalid_instruction!(),
    /* 0x6f */ invalid_instruction!(),
    /* 0x70 */ invalid_instruction!(),
    /* 0x71 */ invalid_instruction!(),
    /* 0x72 */ invalid_instruction!(),
    /* 0x73 */ invalid_instruction!(),
    /* 0x74 */ invalid_instruction!(),
    /* 0x75 */ invalid_instruction!(),
    /* 0x76 */ invalid_instruction!(),
    /* 0x77 */ invalid_instruction!(),
    /* 0x78 */ invalid_instruction!(),
    /* 0x79 */ invalid_instruction!(),
    /* 0x7a */ invalid_instruction!(),
    /* 0x7b */ invalid_instruction!(),
    /* 0x7c */ invalid_instruction!(),
    /* 0x7d */ invalid_instruction!(),
    /* 0x7e */ invalid_instruction!(),
    /* 0x7f */ invalid_instruction!(),
    /* 0x80 */ invalid_instruction!(),
    /* 0x81 */ invalid_instruction!(),
    /* 0x82 */ invalid_instruction!(),
    /* 0x83 */ invalid_instruction!(),
    /* 0x84 */ invalid_instruction!(),
    /* 0x85 */ invalid_instruction!(),
    /* 0x86 */ invalid_instruction!(),
    /* 0x87 */ invalid_instruction!(),
    /* 0x88 */ invalid_instruction!(),
    /* 0x89 */ invalid_instruction!(),
    /* 0x8a */ invalid_instruction!(),
    /* 0x8b */ invalid_instruction!(),
    /* 0x8c */ invalid_instruction!(),
    /* 0x8d */ invalid_instruction!(),
    /* 0x8e */ invalid_instruction!(),
    /* 0x8f */ invalid_instruction!(),
    /* 0x90 */ invalid_instruction!(),
    /* 0x91 */ invalid_instruction!(),
    /* 0x92 */ invalid_instruction!(),
    /* 0x93 */ invalid_instruction!(),
    /* 0x94 */ invalid_instruction!(),
    /* 0x95 */ invalid_instruction!(),
    /* 0x96 */ invalid_instruction!(),
    /* 0x97 */ invalid_instruction!(),
    /* 0x98 */ invalid_instruction!(),
    /* 0x99 */ invalid_instruction!(),
    /* 0x9a */ invalid_instruction!(),
    /* 0x9b */ invalid_instruction!(),
    /* 0x9c */ invalid_instruction!(),
    /* 0x9d */ invalid_instruction!(),
    /* 0x9e */ invalid_instruction!(),
    /* 0x9f */ invalid_instruction!(),
    /* 0xa0 */ invalid_instruction!(),
    /* 0xa1 */ invalid_instruction!(),
    /* 0xa2 */ invalid_instruction!(),
    /* 0xa3 */ invalid_instruction!(),
    /* 0xa4 */ invalid_instruction!(),
    /* 0xa5 */ invalid_instruction!(),
    /* 0xa6 */ invalid_instruction!(),
    /* 0xa7 */ invalid_instruction!(),
    /* 0xa8 */ invalid_instruction!(),
    /* 0xa9 */ invalid_instruction!(),
    /* 0xaa */ invalid_instruction!(),
    /* 0xab */ invalid_instruction!(),
    /* 0xac */ invalid_instruction!(),
    /* 0xad */ invalid_instruction!(),
    /* 0xae */ invalid_instruction!(),
    /* 0xaf */ xor_r8_r8_instruction!(A, A),
    /* 0xb0 */ invalid_instruction!(),
    /* 0xb1 */ invalid_instruction!(),
    /* 0xb2 */ invalid_instruction!(),
    /* 0xb3 */ invalid_instruction!(),
    /* 0xb4 */ invalid_instruction!(),
    /* 0xb5 */ invalid_instruction!(),
    /* 0xb6 */ invalid_instruction!(),
    /* 0xb7 */ invalid_instruction!(),
    /* 0xb8 */ invalid_instruction!(),
    /* 0xb9 */ invalid_instruction!(),
    /* 0xba */ invalid_instruction!(),
    /* 0xbb */ invalid_instruction!(),
    /* 0xbc */ invalid_instruction!(),
    /* 0xbd */ invalid_instruction!(),
    /* 0xbe */ invalid_instruction!(),
    /* 0xbf */ invalid_instruction!(),
    /* 0xc0 */ invalid_instruction!(),
    /* 0xc1 */ invalid_instruction!(),
    /* 0xc2 */ invalid_instruction!(),
    /* 0xc3 */ invalid_instruction!(),
    /* 0xc4 */ invalid_instruction!(),
    /* 0xc5 */ invalid_instruction!(),
    /* 0xc6 */ invalid_instruction!(),
    /* 0xc7 */ invalid_instruction!(),
    /* 0xc8 */ invalid_instruction!(),
    /* 0xc9 */ invalid_instruction!(),
    /* 0xca */ invalid_instruction!(),
    /* 0xcb */ invalid_instruction!(),
    /* 0xcc */ invalid_instruction!(),
    /* 0xcd */ invalid_instruction!(),
    /* 0xce */ invalid_instruction!(),
    /* 0xcf */ invalid_instruction!(),
    /* 0xd0 */ invalid_instruction!(),
    /* 0xd1 */ invalid_instruction!(),
    /* 0xd2 */ invalid_instruction!(),
    /* 0xd3 */ invalid_instruction!(),
    /* 0xd4 */ invalid_instruction!(),
    /* 0xd5 */ invalid_instruction!(),
    /* 0xd6 */ invalid_instruction!(),
    /* 0xd7 */ invalid_instruction!(),
    /* 0xd8 */ invalid_instruction!(),
    /* 0xd9 */ invalid_instruction!(),
    /* 0xda */ invalid_instruction!(),
    /* 0xdb */ invalid_instruction!(),
    /* 0xdc */ invalid_instruction!(),
    /* 0xdd */ invalid_instruction!(),
    /* 0xde */ invalid_instruction!(),
    /* 0xdf */ invalid_instruction!(),
    /* 0xe0 */ invalid_instruction!(),
    /* 0xe1 */ invalid_instruction!(),
    /* 0xe2 */ invalid_instruction!(),
    /* 0xe3 */ invalid_instruction!(),
    /* 0xe4 */ invalid_instruction!(),
    /* 0xe5 */ invalid_instruction!(),
    /* 0xe6 */ invalid_instruction!(),
    /* 0xe7 */ invalid_instruction!(),
    /* 0xe8 */ invalid_instruction!(),
    /* 0xe9 */ invalid_instruction!(),
    /* 0xea */ invalid_instruction!(),
    /* 0xeb */ invalid_instruction!(),
    /* 0xec */ invalid_instruction!(),
    /* 0xed */ invalid_instruction!(),
    /* 0xee */ invalid_instruction!(),
    /* 0xef */ invalid_instruction!(),
    /* 0xf0 */ invalid_instruction!(),
    /* 0xf1 */ invalid_instruction!(),
    /* 0xf2 */ invalid_instruction!(),
    /* 0xf3 */ invalid_instruction!(),
    /* 0xf4 */ invalid_instruction!(),
    /* 0xf5 */ invalid_instruction!(),
    /* 0xf6 */ invalid_instruction!(),
    /* 0xf7 */ invalid_instruction!(),
    /* 0xf8 */ invalid_instruction!(),
    /* 0xf9 */ invalid_instruction!(),
    /* 0xfa */ invalid_instruction!(),
    /* 0xfb */ invalid_instruction!(),
    /* 0xfc */ invalid_instruction!(),
    /* 0xfd */ invalid_instruction!(),
    /* 0xfe */ invalid_instruction!(),
    /* 0xff */ invalid_instruction!(),
];

// TODO: Remove the option as this can't fail
const CB_PREFIXED_INSTRUCTIONS_TABLE: [Option<Instruction>; 0x100] = [
    /* 0x00 */ invalid_instruction!(),
    /* 0x01 */ invalid_instruction!(),
    /* 0x02 */ invalid_instruction!(),
    /* 0x03 */ invalid_instruction!(),
    /* 0x04 */ invalid_instruction!(),
    /* 0x05 */ invalid_instruction!(),
    /* 0x06 */ invalid_instruction!(),
    /* 0x07 */ invalid_instruction!(),
    /* 0x08 */ invalid_instruction!(),
    /* 0x09 */ invalid_instruction!(),
    /* 0x0a */ invalid_instruction!(),
    /* 0x0b */ invalid_instruction!(),
    /* 0x0c */ invalid_instruction!(),
    /* 0x0d */ invalid_instruction!(),
    /* 0x0e */ invalid_instruction!(),
    /* 0x0f */ invalid_instruction!(),
    /* 0x10 */ invalid_instruction!(),
    /* 0x11 */ invalid_instruction!(),
    /* 0x12 */ invalid_instruction!(),
    /* 0x13 */ invalid_instruction!(),
    /* 0x14 */ invalid_instruction!(),
    /* 0x15 */ invalid_instruction!(),
    /* 0x16 */ invalid_instruction!(),
    /* 0x17 */ invalid_instruction!(),
    /* 0x18 */ invalid_instruction!(),
    /* 0x19 */ invalid_instruction!(),
    /* 0x1a */ invalid_instruction!(),
    /* 0x1b */ invalid_instruction!(),
    /* 0x1c */ invalid_instruction!(),
    /* 0x1d */ invalid_instruction!(),
    /* 0x1e */ invalid_instruction!(),
    /* 0x1f */ invalid_instruction!(),
    /* 0x20 */ invalid_instruction!(),
    /* 0x21 */ invalid_instruction!(),
    /* 0x22 */ invalid_instruction!(),
    /* 0x23 */ invalid_instruction!(),
    /* 0x24 */ invalid_instruction!(),
    /* 0x25 */ invalid_instruction!(),
    /* 0x26 */ invalid_instruction!(),
    /* 0x27 */ invalid_instruction!(),
    /* 0x28 */ invalid_instruction!(),
    /* 0x29 */ invalid_instruction!(),
    /* 0x2a */ invalid_instruction!(),
    /* 0x2b */ invalid_instruction!(),
    /* 0x2c */ invalid_instruction!(),
    /* 0x2d */ invalid_instruction!(),
    /* 0x2e */ invalid_instruction!(),
    /* 0x2f */ invalid_instruction!(),
    /* 0x30 */ invalid_instruction!(),
    /* 0x31 */ invalid_instruction!(),
    /* 0x32 */ invalid_instruction!(),
    /* 0x33 */ invalid_instruction!(),
    /* 0x34 */ invalid_instruction!(),
    /* 0x35 */ invalid_instruction!(),
    /* 0x36 */ invalid_instruction!(),
    /* 0x37 */ invalid_instruction!(),
    /* 0x38 */ invalid_instruction!(),
    /* 0x39 */ invalid_instruction!(),
    /* 0x3a */ invalid_instruction!(),
    /* 0x3b */ invalid_instruction!(),
    /* 0x3c */ invalid_instruction!(),
    /* 0x3d */ invalid_instruction!(),
    /* 0x3e */ invalid_instruction!(),
    /* 0x3f */ invalid_instruction!(),
    /* 0x40 */ invalid_instruction!(),
    /* 0x41 */ invalid_instruction!(),
    /* 0x42 */ invalid_instruction!(),
    /* 0x43 */ invalid_instruction!(),
    /* 0x44 */ invalid_instruction!(),
    /* 0x45 */ invalid_instruction!(),
    /* 0x46 */ invalid_instruction!(),
    /* 0x47 */ invalid_instruction!(),
    /* 0x48 */ invalid_instruction!(),
    /* 0x49 */ invalid_instruction!(),
    /* 0x4a */ invalid_instruction!(),
    /* 0x4b */ invalid_instruction!(),
    /* 0x4c */ invalid_instruction!(),
    /* 0x4d */ invalid_instruction!(),
    /* 0x4e */ invalid_instruction!(),
    /* 0x4f */ invalid_instruction!(),
    /* 0x50 */ invalid_instruction!(),
    /* 0x51 */ invalid_instruction!(),
    /* 0x52 */ invalid_instruction!(),
    /* 0x53 */ invalid_instruction!(),
    /* 0x54 */ invalid_instruction!(),
    /* 0x55 */ invalid_instruction!(),
    /* 0x56 */ invalid_instruction!(),
    /* 0x57 */ invalid_instruction!(),
    /* 0x58 */ invalid_instruction!(),
    /* 0x59 */ invalid_instruction!(),
    /* 0x5a */ invalid_instruction!(),
    /* 0x5b */ invalid_instruction!(),
    /* 0x5c */ invalid_instruction!(),
    /* 0x5d */ invalid_instruction!(),
    /* 0x5e */ invalid_instruction!(),
    /* 0x5f */ invalid_instruction!(),
    /* 0x60 */ invalid_instruction!(),
    /* 0x61 */ invalid_instruction!(),
    /* 0x62 */ invalid_instruction!(),
    /* 0x63 */ invalid_instruction!(),
    /* 0x64 */ invalid_instruction!(),
    /* 0x65 */ invalid_instruction!(),
    /* 0x66 */ invalid_instruction!(),
    /* 0x67 */ invalid_instruction!(),
    /* 0x68 */ invalid_instruction!(),
    /* 0x69 */ invalid_instruction!(),
    /* 0x6a */ invalid_instruction!(),
    /* 0x6b */ invalid_instruction!(),
    /* 0x6c */ invalid_instruction!(),
    /* 0x6d */ invalid_instruction!(),
    /* 0x6e */ invalid_instruction!(),
    /* 0x6f */ invalid_instruction!(),
    /* 0x70 */ invalid_instruction!(),
    /* 0x71 */ invalid_instruction!(),
    /* 0x72 */ invalid_instruction!(),
    /* 0x73 */ invalid_instruction!(),
    /* 0x74 */ invalid_instruction!(),
    /* 0x75 */ invalid_instruction!(),
    /* 0x76 */ invalid_instruction!(),
    /* 0x77 */ invalid_instruction!(),
    /* 0x78 */ invalid_instruction!(),
    /* 0x79 */ invalid_instruction!(),
    /* 0x7a */ invalid_instruction!(),
    /* 0x7b */ invalid_instruction!(),
    /* 0x7c */ invalid_instruction!(),
    /* 0x7d */ invalid_instruction!(),
    /* 0x7e */ invalid_instruction!(),
    /* 0x7f */ invalid_instruction!(),
    /* 0x80 */ invalid_instruction!(),
    /* 0x81 */ invalid_instruction!(),
    /* 0x82 */ invalid_instruction!(),
    /* 0x83 */ invalid_instruction!(),
    /* 0x84 */ invalid_instruction!(),
    /* 0x85 */ invalid_instruction!(),
    /* 0x86 */ invalid_instruction!(),
    /* 0x87 */ invalid_instruction!(),
    /* 0x88 */ invalid_instruction!(),
    /* 0x89 */ invalid_instruction!(),
    /* 0x8a */ invalid_instruction!(),
    /* 0x8b */ invalid_instruction!(),
    /* 0x8c */ invalid_instruction!(),
    /* 0x8d */ invalid_instruction!(),
    /* 0x8e */ invalid_instruction!(),
    /* 0x8f */ invalid_instruction!(),
    /* 0x90 */ invalid_instruction!(),
    /* 0x91 */ invalid_instruction!(),
    /* 0x92 */ invalid_instruction!(),
    /* 0x93 */ invalid_instruction!(),
    /* 0x94 */ invalid_instruction!(),
    /* 0x95 */ invalid_instruction!(),
    /* 0x96 */ invalid_instruction!(),
    /* 0x97 */ invalid_instruction!(),
    /* 0x98 */ invalid_instruction!(),
    /* 0x99 */ invalid_instruction!(),
    /* 0x9a */ invalid_instruction!(),
    /* 0x9b */ invalid_instruction!(),
    /* 0x9c */ invalid_instruction!(),
    /* 0x9d */ invalid_instruction!(),
    /* 0x9e */ invalid_instruction!(),
    /* 0x9f */ invalid_instruction!(),
    /* 0xa0 */ invalid_instruction!(),
    /* 0xa1 */ invalid_instruction!(),
    /* 0xa2 */ invalid_instruction!(),
    /* 0xa3 */ invalid_instruction!(),
    /* 0xa4 */ invalid_instruction!(),
    /* 0xa5 */ invalid_instruction!(),
    /* 0xa6 */ invalid_instruction!(),
    /* 0xa7 */ invalid_instruction!(),
    /* 0xa8 */ invalid_instruction!(),
    /* 0xa9 */ invalid_instruction!(),
    /* 0xaa */ invalid_instruction!(),
    /* 0xab */ invalid_instruction!(),
    /* 0xac */ invalid_instruction!(),
    /* 0xad */ invalid_instruction!(),
    /* 0xae */ invalid_instruction!(),
    /* 0xaf */ invalid_instruction!(),
    /* 0xb0 */ invalid_instruction!(),
    /* 0xb1 */ invalid_instruction!(),
    /* 0xb2 */ invalid_instruction!(),
    /* 0xb3 */ invalid_instruction!(),
    /* 0xb4 */ invalid_instruction!(),
    /* 0xb5 */ invalid_instruction!(),
    /* 0xb6 */ invalid_instruction!(),
    /* 0xb7 */ invalid_instruction!(),
    /* 0xb8 */ invalid_instruction!(),
    /* 0xb9 */ invalid_instruction!(),
    /* 0xba */ invalid_instruction!(),
    /* 0xbb */ invalid_instruction!(),
    /* 0xbc */ invalid_instruction!(),
    /* 0xbd */ invalid_instruction!(),
    /* 0xbe */ invalid_instruction!(),
    /* 0xbf */ invalid_instruction!(),
    /* 0xc0 */ invalid_instruction!(),
    /* 0xc1 */ invalid_instruction!(),
    /* 0xc2 */ invalid_instruction!(),
    /* 0xc3 */ invalid_instruction!(),
    /* 0xc4 */ invalid_instruction!(),
    /* 0xc5 */ invalid_instruction!(),
    /* 0xc6 */ invalid_instruction!(),
    /* 0xc7 */ invalid_instruction!(),
    /* 0xc8 */ invalid_instruction!(),
    /* 0xc9 */ invalid_instruction!(),
    /* 0xca */ invalid_instruction!(),
    /* 0xcb */ invalid_instruction!(),
    /* 0xcc */ invalid_instruction!(),
    /* 0xcd */ invalid_instruction!(),
    /* 0xce */ invalid_instruction!(),
    /* 0xcf */ invalid_instruction!(),
    /* 0xd0 */ invalid_instruction!(),
    /* 0xd1 */ invalid_instruction!(),
    /* 0xd2 */ invalid_instruction!(),
    /* 0xd3 */ invalid_instruction!(),
    /* 0xd4 */ invalid_instruction!(),
    /* 0xd5 */ invalid_instruction!(),
    /* 0xd6 */ invalid_instruction!(),
    /* 0xd7 */ invalid_instruction!(),
    /* 0xd8 */ invalid_instruction!(),
    /* 0xd9 */ invalid_instruction!(),
    /* 0xda */ invalid_instruction!(),
    /* 0xdb */ invalid_instruction!(),
    /* 0xdc */ invalid_instruction!(),
    /* 0xdd */ invalid_instruction!(),
    /* 0xde */ invalid_instruction!(),
    /* 0xdf */ invalid_instruction!(),
    /* 0xe0 */ invalid_instruction!(),
    /* 0xe1 */ invalid_instruction!(),
    /* 0xe2 */ invalid_instruction!(),
    /* 0xe3 */ invalid_instruction!(),
    /* 0xe4 */ invalid_instruction!(),
    /* 0xe5 */ invalid_instruction!(),
    /* 0xe6 */ invalid_instruction!(),
    /* 0xe7 */ invalid_instruction!(),
    /* 0xe8 */ invalid_instruction!(),
    /* 0xe9 */ invalid_instruction!(),
    /* 0xea */ invalid_instruction!(),
    /* 0xeb */ invalid_instruction!(),
    /* 0xec */ invalid_instruction!(),
    /* 0xed */ invalid_instruction!(),
    /* 0xee */ invalid_instruction!(),
    /* 0xef */ invalid_instruction!(),
    /* 0xf0 */ invalid_instruction!(),
    /* 0xf1 */ invalid_instruction!(),
    /* 0xf2 */ invalid_instruction!(),
    /* 0xf3 */ invalid_instruction!(),
    /* 0xf4 */ invalid_instruction!(),
    /* 0xf5 */ invalid_instruction!(),
    /* 0xf6 */ invalid_instruction!(),
    /* 0xf7 */ invalid_instruction!(),
    /* 0xf8 */ invalid_instruction!(),
    /* 0xf9 */ invalid_instruction!(),
    /* 0xfa */ invalid_instruction!(),
    /* 0xfb */ invalid_instruction!(),
    /* 0xfc */ invalid_instruction!(),
    /* 0xfd */ invalid_instruction!(),
    /* 0xfe */ invalid_instruction!(),
    /* 0xff */ invalid_instruction!(),
];
