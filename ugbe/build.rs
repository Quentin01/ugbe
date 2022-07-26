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

/// Decode an instruction depending on its opcode
/// See https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
fn decode_instruction(cb_prefixed: bool, opcode: u8) -> Cow<'static, str> {
    let x = (opcode >> 6) & 0b11;
    let y = (opcode >> 3) & 0b111;
    let z = opcode & 0b111;
    let p = (y >> 1) & 0b11;
    let q = y & 0b1;

    if !cb_prefixed {
        if x == 0 {
            if z == 0 {
                if y == 0 {
                    // TODO nop
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                } else if y == 1 {
                    "&implementation::Ld::<operands::DerefImm16ToU16, operands::SP>::new()".into()
                } else if y == 2 {
                    // TODO STOP
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
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
                    // TODO ADD HL, rp[p]
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
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
                // TODO INC/DEC r16
                format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
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
                // TODO Assorted operations on A
                format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
            } else {
                unreachable!("Z > 7")
            }
        } else if x == 1 {
            if z == 6 && y == 6 {
                // TODO
                format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
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
                if y == 4 {
                    "&implementation::Ld::<operands::DerefImm8, operands::A>::new()".into()
                } else if y == 6 {
                    "&implementation::Ld::<operands::A, operands::DerefImm8>::new()".into()
                } else {
                    // TODO
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                }
            } else if z == 2 {
                if y == 4 {
                    "&implementation::Ld::<operands::DerefC, operands::A>::new()".into()
                } else if y == 6 {
                    "&implementation::Ld::<operands::A, operands::DerefC>::new()".into()
                } else {
                    // TODO
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                }
            } else if z == 3 {
                if y == 1 {
                    // CB prefix already handled by the CPU
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                } else {
                    // TODO
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                }
            } else if z == 4 {
                if (0..3).contains(&y) {
                    format!(
                        "&implementation::Call::<{}, operands::Imm16>::new()",
                        TABLE_CC[y as usize]
                    )
                    .into()
                } else {
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                }
            } else if z == 5 {
                if q == 0 {
                    // TODO push rp2[p]
                    format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                } else if q == 1 {
                    if p == 0 {
                        "&implementation::Call::<condition::None, operands::Imm16>::new()".into()
                    } else {
                        // TODO
                        format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
                    }
                } else {
                    unreachable!("Q > 1")
                }
            } else {
                // TODO
                format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
            }
        } else {
            // TODO
            format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
        }
    } else {
        if x == 0 {
            // TODO rot[y], r[z]
            format!("&implementation::Invalid::<{opcode}, {cb_prefixed}>::new()").into()
        } else if x == 1 {
            format!(
                "&implementation::AluBit::<alu::Bit, {y}, {}>::new()",
                TABLE_R[z as usize]
            )
            .into()
        } else if x == 2 {
            format!(
                "&implementation::AluBit::<alu::Res, {y}, {}>::new()",
                TABLE_R[z as usize]
            )
            .into()
        } else if x == 3 {
            format!(
                "&implementation::AluBit::<alu::Set, {y}, {}>::new()",
                TABLE_R[z as usize]
            )
            .into()
        } else {
            unreachable!("X > 3")
        }
    }
}

fn decode_instructions(cb_prefixed: bool) -> [String; 0xFF] {
    let mut instructions = Vec::with_capacity(0xFF);

    for opcode in 0..0xFF {
        instructions.push(format!(
            "\t/* 0x{opcode:02x} */ {}",
            decode_instruction(cb_prefixed, opcode as u8),
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
            "pub const INSTRUCTIONS_TABLE: [&'static dyn Instruction; 0xFF] = [
{}
];

pub const CB_PREFIXED_INSTRUCTIONS_TABLE: [&'static dyn Instruction; 0xFF] = [
{}
];",
            decode_instructions(false).join(",\n"),
            decode_instructions(true).join(",\n")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
