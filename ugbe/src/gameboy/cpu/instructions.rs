use const_format::concatcp;

mod alu;
mod instructions;
mod machine_cycle;
mod operands;

use alu::{Alu, Alu16, Alu8};
use operands::Operand;

pub use instructions::{Instruction, InstructionState};

macro_rules! not_conditional_operations {
    ($ok:expr) => {
        machine_cycle::Operations::NotConditional($ok)
    };
}

macro_rules! conditional_operations {
    ($cond:ident, $ok:expr, $not_ok:expr) => {
        machine_cycle::Operations::Conditional {
            cond: machine_cycle::Condition::$cond,
            ok: $ok,
            not_ok: $not_ok,
        }
    };
}

const NOTHING: &dyn machine_cycle::Operation = &|_: &mut super::Cpu| {};

const PREFETCH_NEXT_OP: Option<&dyn machine_cycle::MemoryOperationGenerator> =
    Some(&|_: &mut super::Cpu| machine_cycle::MemoryOperation::PrefetchNextOp);

const READ_PC: Option<&dyn machine_cycle::MemoryOperationGenerator> =
    Some(&|cpu: &mut super::Cpu| {
        let pc = cpu.registers.pc;
        cpu.registers.pc = pc.wrapping_add(1);
        machine_cycle::MemoryOperation::Read(pc)
    });

macro_rules! invalid_instruction {
    ($opcode:expr, $cb_prefixed:expr) => {
        instructions::Instruction::new(
            "INVALID",
            not_conditional_operations!(&[machine_cycle::MachineCycle::new(
                &|cpu: &mut super::Cpu| {
                    if let super::State::ExecutingInstruction(instruction_state) = cpu.state {
                        panic!(
                            "Encountered invalid instruction ({}0x{:02x}) at ${:04x}",
                            if $cb_prefixed { "0xCB " } else { "" },
                            $opcode,
                            instruction_state.start_pc()
                        )
                    }
                },
                PREFETCH_NEXT_OP
            )]),
        )
    };
}

macro_rules! nop_instruction {
    () => {
        instructions::Instruction::new(
            "NOP",
            not_conditional_operations!(&[machine_cycle::MachineCycle::new(
                NOTHING,
                PREFETCH_NEXT_OP
            )]),
        )
    };
}

macro_rules! ld8_instruction {
    ($dst:ty, $src:ty) => {
        instructions::Instruction::new(
            concatcp!("LD ", <$dst>::STR, ", ", <$src>::STR),
            not_conditional_operations!({
                match (<$dst>::KIND, <$src>::KIND) {
                    (operands::Kind::Register, operands::Kind::Register) => {
                        // LD r8, r8
                        &[machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| {
                                let value = <$src>::read(cpu);
                                <$dst>::write(cpu, value);
                            },
                            PREFETCH_NEXT_OP,
                        )]
                    }
                    (operands::Kind::Register, operands::Kind::Immediate(8)) => {
                        // LD r8, u8
                        &[
                            machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| {
                                    <$dst>::write(cpu, cpu.data_bus);
                                },
                                PREFETCH_NEXT_OP,
                            ),
                        ]
                    }
                    (operands::Kind::Memory, operands::Kind::Immediate(8)) => {
                        // LD (r16), u8
                        &[
                            machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    machine_cycle::MemoryOperation::Write(
                                        <$dst>::address(cpu),
                                        cpu.data_bus,
                                    )
                                }),
                            ),
                            machine_cycle::MachineCycle::new(NOTHING, PREFETCH_NEXT_OP),
                        ]
                    }
                    (operands::Kind::Register, operands::Kind::Memory) => {
                        // LD r8, (r16)
                        &[
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    machine_cycle::MemoryOperation::Read(<$src>::address(cpu))
                                }),
                            ),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| <$dst>::write(cpu, cpu.data_bus),
                                PREFETCH_NEXT_OP,
                            ),
                        ]
                    }
                    (operands::Kind::Memory, operands::Kind::Register) => {
                        // LD (r16), r8
                        &[
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    let value = <$src>::read(cpu);
                                    machine_cycle::MemoryOperation::Write(
                                        <$dst>::address(cpu),
                                        value,
                                    )
                                }),
                            ),
                            machine_cycle::MachineCycle::new(NOTHING, PREFETCH_NEXT_OP),
                        ]
                    }
                    (operands::Kind::Register, operands::Kind::ImmediateMemory(8)) => {
                        // LD r8, (F00+u8)
                        &[
                            machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    machine_cycle::MemoryOperation::Read(<$src>::address(cpu))
                                }),
                            ),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| <$dst>::write(cpu, cpu.data_bus),
                                PREFETCH_NEXT_OP,
                            ),
                        ]
                    }
                    (operands::Kind::ImmediateMemory(8), operands::Kind::Register) => {
                        // LD (F00+u8), r8
                        &[
                            machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    let value = <$src>::read(cpu);
                                    machine_cycle::MemoryOperation::Write(
                                        <$dst>::address(cpu),
                                        value,
                                    )
                                }),
                            ),
                            machine_cycle::MachineCycle::new(NOTHING, PREFETCH_NEXT_OP),
                        ]
                    }
                    _ => {
                        panic!("Not valid case of LD8")
                    }
                }
            }),
        )
    };
}

macro_rules! ld16_instruction {
    ($dst:ty, $src:ty) => {
        instructions::Instruction::new(
            concatcp!("LD ", <$dst>::STR, ", ", <$src>::STR),
            not_conditional_operations!({
                match (<$dst>::KIND, <$src>::KIND) {
                    (operands::Kind::Register, operands::Kind::Register) => {
                        // LD r8, r8
                        &[machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| {
                                let value = <$src>::read(cpu);
                                <$dst>::write(cpu, value);
                            },
                            PREFETCH_NEXT_OP,
                        )]
                    }
                    (operands::Kind::Register, operands::Kind::Immediate(16)) => {
                        // LD r16, imm16
                        &[
                            machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| <$dst>::write_lsb(cpu, cpu.data_bus),
                                READ_PC,
                            ),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| <$dst>::write_msb(cpu, cpu.data_bus),
                                PREFETCH_NEXT_OP,
                            ),
                        ]
                    }
                    _ => {
                        panic!("Not valid case of LD16")
                    }
                }
            }),
        )
    };
}

macro_rules! alu8_instruction {
    ($alu:ty, $dst:ty, $src:ty) => {
        instructions::Instruction::new(
            concatcp!(<$alu>::STR, " ", <$dst>::STR, ", ", <$src>::STR),
            not_conditional_operations!({
                match (<$dst>::KIND, <$src>::KIND) {
                    (
                        operands::Kind::Register,
                        operands::Kind::Register | operands::Kind::Direct,
                    ) => {
                        // ALU r8, (r8|n)
                        &[machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| {
                                let a = <$dst>::read(cpu);
                                let b = <$src>::read(cpu);
                                let result = <$alu>::execute8(a, b, cpu.registers.cf());

                                if let Some(value) = result.value {
                                    <$dst>::write(cpu, value);
                                }
                                if let Some(zf) = result.zf {
                                    cpu.registers.set_zf(zf);
                                }
                                if let Some(nf) = result.nf {
                                    cpu.registers.set_nf(nf);
                                }
                                if let Some(hf) = result.hf {
                                    cpu.registers.set_hf(hf);
                                }
                                if let Some(cf) = result.cf {
                                    cpu.registers.set_cf(cf);
                                }
                            },
                            PREFETCH_NEXT_OP,
                        )]
                    }
                    (operands::Kind::Register, operands::Kind::Memory | operands::Kind::Direct) => {
                        // ALU r8, ((r16)|n)
                        &[
                            machine_cycle::MachineCycle::new(
                                NOTHING,
                                Some(&|cpu: &mut super::Cpu| {
                                    machine_cycle::MemoryOperation::Read(<$dst>::address(cpu))
                                }),
                            ),
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| {
                                    let a = <$dst>::read(cpu);
                                    let b = cpu.data_bus;
                                    let result = <$alu>::execute8(a, b, cpu.registers.cf());

                                    if let Some(value) = result.value {
                                        <$dst>::write(cpu, value);
                                    }
                                    if let Some(zf) = result.zf {
                                        cpu.registers.set_zf(zf);
                                    }
                                    if let Some(nf) = result.nf {
                                        cpu.registers.set_nf(nf);
                                    }
                                    if let Some(hf) = result.hf {
                                        cpu.registers.set_hf(hf);
                                    }
                                    if let Some(cf) = result.cf {
                                        cpu.registers.set_cf(cf);
                                    }
                                },
                                PREFETCH_NEXT_OP,
                            ),
                        ]
                    }
                    (operands::Kind::Memory, operands::Kind::Direct) => {
                        // ALU (r16), n
                        &[
                            machine_cycle::MachineCycle::new(
                                &|cpu: &mut super::Cpu| {
                                    let a = <$dst>::read(cpu);
                                    let b = cpu.data_bus;
                                    let result = <$alu>::execute8(a, b, cpu.registers.cf());

                                    if let Some(value) = result.value {
                                        cpu.registers.x = value;
                                    }
                                    if let Some(zf) = result.zf {
                                        cpu.registers.set_zf(zf);
                                    }
                                    if let Some(nf) = result.nf {
                                        cpu.registers.set_nf(nf);
                                    }
                                    if let Some(hf) = result.hf {
                                        cpu.registers.set_hf(hf);
                                    }
                                    if let Some(cf) = result.cf {
                                        cpu.registers.set_cf(cf);
                                    }
                                },
                                Some(&|cpu: &mut super::Cpu| {
                                    machine_cycle::MemoryOperation::Write(
                                        <$dst>::address(cpu),
                                        cpu.registers.x,
                                    )
                                }),
                            ),
                            machine_cycle::MachineCycle::new(NOTHING, PREFETCH_NEXT_OP),
                        ]
                    }
                    _ => {
                        panic!("Not valid case of ALU8")
                    }
                }
            }),
        )
    };
}

macro_rules! jr_instruction {
    ($off:ty) => {
        // JR i8
        instructions::Instruction::new(
            concatcp!("JR ", <$off>::STR),
            not_conditional_operations!({
                &[
                    machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                    machine_cycle::MachineCycle::new(
                        &|cpu: &mut super::Cpu| {
                            cpu.registers.set_xy(
                                ((cpu.registers.pc as i32) + ((cpu.data_bus as i8) as i32)) as u16,
                            );
                        },
                        None,
                    ),
                    machine_cycle::MachineCycle::new(
                        &|cpu: &mut super::Cpu| cpu.registers.pc = cpu.registers.xy(),
                        PREFETCH_NEXT_OP,
                    ),
                ]
            }),
        )
    };
    ($cond:ident, $off:ty) => {
        // JR CC, i8
        instructions::Instruction::new(
            concatcp!("JR ", stringify!($cond), ", ", <$off>::STR),
            conditional_operations!(
                $cond,
                {
                    &[
                        machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                        machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| {
                                cpu.registers.set_xy(
                                    ((cpu.registers.pc as i32) + ((cpu.data_bus as i8) as i32))
                                        as u16,
                                );
                            },
                            None,
                        ),
                        machine_cycle::MachineCycle::new(NOTHING, None),
                        machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| cpu.registers.pc = cpu.registers.xy(),
                            PREFETCH_NEXT_OP,
                        ),
                    ]
                },
                {
                    &[
                        machine_cycle::MachineCycle::new(NOTHING, READ_PC),
                        machine_cycle::MachineCycle::new(
                            &|cpu: &mut super::Cpu| {
                                cpu.registers.set_xy(
                                    ((cpu.registers.pc as i32) + ((cpu.data_bus as i8) as i32))
                                        as u16,
                                );
                            },
                            None,
                        ),
                        machine_cycle::MachineCycle::new(NOTHING, PREFETCH_NEXT_OP),
                    ]
                }
            ),
        )
    };
}

macro_rules! cb_prefix {
    () => {
        instructions::Instruction::new(
            "CB",
            not_conditional_operations!(&[machine_cycle::MachineCycle::new(
                NOTHING,
                Some(&|_: &mut super::Cpu| machine_cycle::MemoryOperation::PrefetchNextCbOp)
            )]),
        )
    };
}

include!(concat!(env!("OUT_DIR"), "/instructions_gen.rs"));
