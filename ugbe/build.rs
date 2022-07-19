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

const TABLE_CC: [&str; 4] = ["NZ", "Z", "NC", "C"];

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
                    "nop_instruction!()".into()
                } else if y == 1 {
                    // TODO LD (nn), SP
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                } else if y == 2 {
                    // TODO STOP
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                } else if y == 3 {
                    "jr_instruction!(operands::Off8)".into()
                } else if (4..=7).contains(&y) {
                    format!(
                        "jr_instruction!({}, operands::Off8)",
                        TABLE_CC[(y - 4) as usize]
                    )
                    .into()
                } else {
                    unreachable!("Y > 7")
                }
            } else if z == 1 {
                if q == 0 {
                    // TODO LD rp[p], nn
                    format!(
                        "ld16_instruction!({}, operands::Imm16)",
                        TABLE_RP[p as usize]
                    )
                    .into()
                } else if q == 1 {
                    // TODO ADD HL, rp[p]
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                } else {
                    unreachable!("Q > 1")
                }
            } else if z == 2 {
                if q == 0 {
                    if p == 0 {
                        "ld8_instruction!(operands::DerefBC, operands::A)".into()
                    } else if p == 1 {
                        "ld8_instruction!(operands::DerefDE, operands::A)".into()
                    } else if p == 2 {
                        "ld8_instruction!(operands::DerefIncHL, operands::A)".into()
                    } else if p == 3 {
                        "ld8_instruction!(operands::DerefDecHL, operands::A)".into()
                    } else {
                        unreachable!("P > 4")
                    }
                } else if q == 1 {
                    if p == 0 {
                        "ld8_instruction!(operands::A, operands::DerefBC)".into()
                    } else if p == 1 {
                        "ld8_instruction!(operands::A, operands::DerefDE)".into()
                    } else if p == 2 {
                        "ld8_instruction!(operands::A, operands::DerefIncHL)".into()
                    } else if p == 3 {
                        "ld8_instruction!(operands::A, operands::DerefDecHL)".into()
                    } else {
                        unreachable!("P > 4")
                    }
                } else {
                    unreachable!("Q > 1")
                }
            } else if z == 6 {
                format!("ld8_instruction!({}, operands::Imm8)", TABLE_R[y as usize]).into()
            } else {
                // TODO
                format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
            }
        } else if x == 1 {
            if z == 6 && y == 6 {
                // TODO
                format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
            } else {
                format!(
                    "ld8_instruction!({}, {})",
                    TABLE_R[y as usize], TABLE_R[z as usize]
                )
                .into()
            }
        } else if x == 2 {
            format!(
                "alu8_instruction!({}, operands::A, {})",
                TABLE_ALU[y as usize], TABLE_R[z as usize]
            )
            .into()
        } else if x == 3 {
            if z == 0 {
                if y == 4 {
                    "ld8_instruction!(operands::DerefHighImm8, operands::A)".into()
                } else if y == 6 {
                    "ld8_instruction!(operands::A, operands::DerefHighImm8)".into()
                } else {
                    // TODO
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                }
            } else if z == 2 {
                if y == 4 {
                    "ld8_instruction!(operands::DerefHighC, operands::A)".into()
                } else if y == 6 {
                    "ld8_instruction!(operands::A, operands::DerefHighC)".into()
                } else {
                    // TODO
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                }
            } else if z == 3 {
                if y == 1 {
                    "cb_prefix!()".into()
                } else {
                    // TODO
                    format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
                }
            } else {
                // TODO
                format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
            }
        } else {
            // TODO
            format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
        }
    } else {
        if x == 0 {
            // TODO rot[y], r[z]
            format!("invalid_instruction!({opcode}, {cb_prefixed})").into()
        } else if x == 1 {
            format!(
                "alu8_instruction!(alu::Bit, {}, operands::Direct<{y}>)",
                TABLE_R[z as usize]
            )
            .into()
        } else if x == 2 {
            format!(
                "alu8_instruction!(alu::Res, {}, operands::Direct<{y}>)",
                TABLE_R[z as usize]
            )
            .into()
        } else if x == 3 {
            format!(
                "alu8_instruction!(alu::Set, {}, operands::Direct<{y}>)",
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
            "pub const INSTRUCTIONS_TABLE: [instructions::Instruction; 0xFF] = [
{}
];

pub const CB_PREFIXED_INSTRUCTIONS_TABLE: [instructions::Instruction; 0xFF] = [
{}
];",
            decode_instructions(false).join(",\n"),
            decode_instructions(true).join(",\n")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
