use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::Path;

const TABLE_R: [&str; 8] = [
    "operands::B",
    "operands::C",
    "operands::D",
    "operands::E",
    "operands::H",
    "operands::L",
    "operands::DerefHL",
    "operands::A",
];

const TABLE_RP: [&str; 4] = [
    "operands::BC",
    "operands::DE",
    "operands::HL",
    "operands::SP",
];

const TABLE_RP2: [&str; 4] = [
    "operands::BC",
    "operands::DE",
    "operands::HL",
    "operands::AF",
];

const TABLE_CC: [&str; 4] = [
    "condition::NZ",
    "condition::Z",
    "condition::NC",
    "condition::C",
];

const TABLE_ALU: [&str; 8] = [
    "alu::Add", "alu::Adc", "alu::Sub", "alu::Sbc", "alu::And", "alu::Xor", "alu::Or", "alu::Cp",
];

const TABLE_ROT: [&str; 8] = [
    "alu::Rlc",
    "alu::Rrc",
    "alu::Rl",
    "alu::Rr",
    "alu::Sla",
    "alu::Sra",
    "alu::Swap",
    "alu::Srl",
];

/// Decode an instruction depending on its opcode
/// See https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
fn decode_instruction(opcode: u8) -> Cow<'static, str> {
    let x = (opcode >> 6) & 0b11;
    let y = (opcode >> 3) & 0b111;
    let z = opcode & 0b111;
    let p = (y >> 1) & 0b11;
    let q = y & 0b1;

    if x == 0 {
        if z == 0 {
            if y == 0 {
                "&implementation::Nop::new()".into()
            } else if y == 1 {
                "&implementation::Ld::<operands::DerefImm16ToU16, operands::SP>::new()".into()
            } else if y == 2 {
                // TODO: STOP
                format!("&implementation::Invalid::<{opcode}, false>::new()").into()
            } else if y == 3 {
                "&implementation::Jr::<condition::None, operands::Off8>::new()".into()
            } else if (4..=7).contains(&y) {
                format!(
                    "&implementation::Jr::<{}, operands::Off8>::new()",
                    TABLE_CC[(y - 4) as usize]
                )
                .into()
            } else {
                unreachable!("Y > 7")
            }
        } else if z == 1 {
            if q == 0 {
                format!(
                    "&implementation::Ld::<{}, operands::Imm16>::new()",
                    TABLE_RP[p as usize]
                )
                .into()
            } else if q == 1 {
                format!(
                    "&implementation::AluTwo::<alu::Add, operands::HL, {}>::new()",
                    TABLE_RP[p as usize]
                )
                .into()
            } else {
                unreachable!("Q > 1")
            }
        } else if z == 2 {
            if q == 0 {
                if p == 0 {
                    "&implementation::Ld::<operands::DerefBC, operands::A>::new()".into()
                } else if p == 1 {
                    "&implementation::Ld::<operands::DerefDE, operands::A>::new()".into()
                } else if p == 2 {
                    "&implementation::Ld::<operands::DerefIncHL, operands::A>::new()".into()
                } else if p == 3 {
                    "&implementation::Ld::<operands::DerefDecHL, operands::A>::new()".into()
                } else {
                    unreachable!("P > 4")
                }
            } else if q == 1 {
                if p == 0 {
                    "&implementation::Ld::<operands::A, operands::DerefBC>::new()".into()
                } else if p == 1 {
                    "&implementation::Ld::<operands::A, operands::DerefDE>::new()".into()
                } else if p == 2 {
                    "&implementation::Ld::<operands::A, operands::DerefIncHL>::new()".into()
                } else if p == 3 {
                    "&implementation::Ld::<operands::A, operands::DerefDecHL>::new()".into()
                } else {
                    unreachable!("P > 4")
                }
            } else {
                unreachable!("Q > 1")
            }
        } else if z == 3 {
            if q == 0 {
                format!(
                    "&implementation::AluOne::<alu::Inc, {}>::new()",
                    TABLE_RP[p as usize]
                )
                .into()
            } else if q == 1 {
                format!(
                    "&implementation::AluOne::<alu::Dec, {}>::new()",
                    TABLE_RP[p as usize]
                )
                .into()
            } else {
                unreachable!("Q > 1")
            }
        } else if z == 4 {
            format!(
                "&implementation::AluOne::<alu::Inc, {}>::new()",
                TABLE_R[y as usize]
            )
            .into()
        } else if z == 5 {
            format!(
                "&implementation::AluOne::<alu::Dec, {}>::new()",
                TABLE_R[y as usize]
            )
            .into()
        } else if z == 6 {
            format!(
                "&implementation::Ld::<{}, operands::Imm8>::new()",
                TABLE_R[y as usize]
            )
            .into()
        } else if z == 7 {
            #[allow(clippy::if_same_then_else)]
            if y == 0 {
                "&implementation::AluOne::<alu::RlcA, operands::A>::new()".into()
            } else if y == 1 {
                "&implementation::AluOne::<alu::RrcA, operands::A>::new()".into()
            } else if y == 2 {
                "&implementation::AluOne::<alu::RlA, operands::A>::new()".into()
            } else if y == 3 {
                "&implementation::AluOne::<alu::RrA, operands::A>::new()".into()
            } else if y == 4 {
                "&implementation::AluOne::<alu::Daa, operands::A>::new()".into()
            } else if y == 5 {
                "&implementation::AluOne::<alu::Cpl, operands::A>::new()".into()
            } else if y == 6 {
                "&implementation::AluOne::<alu::Scf, operands::A>::new()".into()
            } else if y == 7 {
                "&implementation::AluOne::<alu::Ccf, operands::A>::new()".into()
            } else {
                unreachable!("Y > 7")
            }
        } else {
            unreachable!("Z > 7")
        }
    } else if x == 1 {
        if z == 6 && y == 6 {
            "&implementation::Halt::new()".into()
        } else {
            format!(
                "&implementation::Ld::<{}, {}>::new()",
                TABLE_R[y as usize], TABLE_R[z as usize]
            )
            .into()
        }
    } else if x == 2 {
        format!(
            "&implementation::AluTwo::<{}, operands::A, {}>::new()",
            TABLE_ALU[y as usize], TABLE_R[z as usize]
        )
        .into()
    } else if x == 3 {
        if z == 0 {
            if (0..=3).contains(&y) {
                format!("&implementation::Ret::<{}>::new()", TABLE_CC[y as usize]).into()
            } else if y == 4 {
                "&implementation::Ld::<operands::DerefImm8, operands::A>::new()".into()
            } else if y == 5 {
                "&implementation::AluTwo::<alu::Add, operands::SP, operands::Off8>::new()".into()
            } else if y == 6 {
                "&implementation::Ld::<operands::A, operands::DerefImm8>::new()".into()
            } else if y == 7 {
                "&implementation::Ld::<operands::HL, operands::SPPlusOff8>::new()".into()
            } else {
                unreachable!("Y > 7")
            }
        } else if z == 1 {
            if q == 0 {
                format!("&implementation::Pop::<{}>::new()", TABLE_RP2[p as usize]).into()
            } else if q == 1 {
                #[allow(clippy::if_same_then_else)]
                if p == 0 {
                    "&implementation::Ret::<condition::None>::new()".into()
                } else if p == 1 {
                    "&implementation::Ret::<condition::None, true>::new()".into()
                } else if p == 2 {
                    "&implementation::Jp::<condition::None, operands::HL, false>::new()".into()
                } else if p == 3 {
                    "&implementation::Ld::<operands::SP, operands::HL, true>::new()".into()
                } else {
                    unreachable!("P > 3")
                }
            } else {
                unreachable!("Q > 1")
            }
        } else if z == 2 {
            if (0..=3).contains(&y) {
                format!(
                    "&implementation::Jp::<{}, operands::Imm16>::new()",
                    TABLE_CC[y as usize]
                )
                .into()
            } else if y == 4 {
                "&implementation::Ld::<operands::DerefC, operands::A>::new()".into()
            } else if y == 5 {
                "&implementation::Ld::<operands::DerefImm16, operands::A>::new()".into()
            } else if y == 6 {
                "&implementation::Ld::<operands::A, operands::DerefC>::new()".into()
            } else if y == 7 {
                "&implementation::Ld::<operands::A, operands::DerefImm16>::new()".into()
            } else {
                unreachable!("Y > 7")
            }
        } else if z == 3 {
            #[allow(clippy::if_same_then_else)]
            if y == 0 {
                "&implementation::Jp::<condition::None, operands::Imm16>::new()".into()
            } else if y == 1 {
                // CB prefix already handled by the CPU
                format!("&implementation::Invalid::<{opcode}, false>::new()").into()
            } else if (2..=5).contains(&y) {
                // Invalid opcodes
                format!("&implementation::Invalid::<{opcode}, false>::new()").into()
            } else if y == 6 {
                "&implementation::DI::new()".into()
            } else if y == 7 {
                "&implementation::EI::new()".into()
            } else {
                unreachable!("Y > 7")
            }
        } else if z == 4 {
            if (0..=3).contains(&y) {
                format!(
                    "&implementation::Call::<{}, operands::Imm16>::new()",
                    TABLE_CC[y as usize]
                )
                .into()
            } else {
                // Invalid opcodes
                format!("&implementation::Invalid::<{opcode}, false>::new()").into()
            }
        } else if z == 5 {
            if q == 0 {
                format!("&implementation::Push::<{}>::new()", TABLE_RP2[p as usize]).into()
            } else if q == 1 {
                if p == 0 {
                    "&implementation::Call::<condition::None, operands::Imm16>::new()".into()
                } else {
                    // Invalid opcodes
                    format!("&implementation::Invalid::<{opcode}, false>::new()").into()
                }
            } else {
                unreachable!("Q > 1")
            }
        } else if z == 6 {
            format!(
                "&implementation::AluTwo::<{}, operands::A, operands::Imm8>::new()",
                TABLE_ALU[y as usize]
            )
            .into()
        } else {
            format!("&implementation::Rst::<{}>::new()", y * 8).into()
        }
    } else {
        unreachable!("X > 3")
    }
}

/// Decode a CB prefixed instruction depending on its opcode
/// See https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
fn decode_cb_instruction(opcode: u8) -> Cow<'static, str> {
    let x = (opcode >> 6) & 0b11;
    let y = (opcode >> 3) & 0b111;
    let z = opcode & 0b111;

    match x {
        0 => format!(
            "&implementation::AluOne::<{}, {}>::new()",
            TABLE_ROT[y as usize], TABLE_R[z as usize]
        )
        .into(),
        1 => format!(
            "&implementation::AluBit::<alu::Bit, {y}, {}>::new()",
            TABLE_R[z as usize]
        )
        .into(),
        2 => format!(
            "&implementation::AluBit::<alu::Res, {y}, {}>::new()",
            TABLE_R[z as usize]
        )
        .into(),
        3 => format!(
            "&implementation::AluBit::<alu::Set, {y}, {}>::new()",
            TABLE_R[z as usize]
        )
        .into(),
        _ => unreachable!("X > 3"),
    }
}

fn decode_instructions(cb_prefixed: bool) -> [String; 256] {
    let mut instructions = Vec::with_capacity(256);

    for opcode in 0..256 {
        instructions.push(format!(
            "\t/* 0x{opcode:02x} */ {}",
            if cb_prefixed {
                decode_cb_instruction(opcode as u8)
            } else {
                decode_instruction(opcode as u8)
            }
        ));
    }

    instructions.try_into().unwrap()
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("instructions_gen.rs");

    fs::write(
        &dest_path,
        format!(
            "pub const INSTRUCTIONS_TABLE: [&'static dyn Instruction; 256] = [
{}
];

pub const CB_PREFIXED_INSTRUCTIONS_TABLE: [&'static dyn Instruction; 256] = [
{}
];",
            decode_instructions(false).join(",\n"),
            decode_instructions(true).join(",\n")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
