pub trait ALUOp {
    const STR: &'static str;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ALUOpResult<Value> {
    pub value: Option<Value>,
    pub zf: Option<bool>,
    pub nf: Option<bool>,
    pub hf: Option<bool>,
    pub cf: Option<bool>,
}

pub trait ALUOneOp<Value>: ALUOp {
    fn execute(value: Value, nf: bool, hf: bool, cf: bool) -> ALUOpResult<Value>;
}

pub trait ALUTwoOp<DstValue, SrcValue>: ALUOp {
    fn execute(a: DstValue, b: SrcValue, cf: bool) -> ALUOpResult<DstValue>;
}

pub trait AluBitOp<const BIT_POS: u8>: ALUOp {
    fn execute(value: u8) -> ALUOpResult<u8>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Inc {}

impl ALUOp for Inc {
    const STR: &'static str = "INC";
}

impl ALUOneOp<u8> for Inc {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value.wrapping_add(1);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(value & 0xF == 0xF),
            cf: None,
        }
    }
}

impl ALUOneOp<u16> for Inc {
    fn execute(value: u16, _: bool, _: bool, _: bool) -> ALUOpResult<u16> {
        let new_value = value.wrapping_add(1);

        ALUOpResult {
            value: Some(new_value),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dec {}

impl ALUOp for Dec {
    const STR: &'static str = "DEC";
}

impl ALUOneOp<u8> for Dec {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value.wrapping_sub(1);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(true),
            hf: Some(value & 0xF == 0x0),
            cf: None,
        }
    }
}

impl ALUOneOp<u16> for Dec {
    fn execute(value: u16, _: bool, _: bool, _: bool) -> ALUOpResult<u16> {
        let new_value = value.wrapping_sub(1);

        ALUOpResult {
            value: Some(new_value),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RLC {}

impl ALUOp for RLC {
    const STR: &'static str = "RLC";
}

impl ALUOneOp<u8> for RLC {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value.rotate_left(1);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RL {}

impl ALUOp for RL {
    const STR: &'static str = "RL";
}

impl ALUOneOp<u8> for RL {
    fn execute(value: u8, _: bool, _: bool, cf: bool) -> ALUOpResult<u8> {
        let new_value = (value << 1) | (cf as u8);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RRC {}

impl ALUOp for RRC {
    const STR: &'static str = "RRC";
}

impl ALUOneOp<u8> for RRC {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value.rotate_right(1);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RR {}

impl ALUOp for RR {
    const STR: &'static str = "RR";
}

impl ALUOneOp<u8> for RR {
    fn execute(value: u8, _: bool, _: bool, cf: bool) -> ALUOpResult<u8> {
        let new_value = (value >> 1) | ((cf as u8) << 7);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SLA {}

impl ALUOp for SLA {
    const STR: &'static str = "SLA";
}

impl ALUOneOp<u8> for SLA {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value << 1;

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRA {}

impl ALUOp for SRA {
    const STR: &'static str = "SRA";
}

impl ALUOneOp<u8> for SRA {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value >> 1 | (value & 0x80);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Swap {}

impl ALUOp for Swap {
    const STR: &'static str = "SWAP";
}

impl ALUOneOp<u8> for Swap {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = (value >> 4) | (value << 4);

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRL {}

impl ALUOp for SRL {
    const STR: &'static str = "SRL";
}

impl ALUOneOp<u8> for SRL {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let new_value = value >> 1;

        ALUOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 == 0x1),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RLCA {}

impl ALUOp for RLCA {
    const STR: &'static str = "RLCA";
}

impl ALUOneOp<u8> for RLCA {
    fn execute(value: u8, nf: bool, hf: bool, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            zf: Some(false),
            ..RLC::execute(value, nf, hf, cf)
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RRCA {}

impl ALUOp for RRCA {
    const STR: &'static str = "RRCA";
}

impl ALUOneOp<u8> for RRCA {
    fn execute(value: u8, nf: bool, hf: bool, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            zf: Some(false),
            ..RRC::execute(value, nf, hf, cf)
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RLA {}

impl ALUOp for RLA {
    const STR: &'static str = "RLA";
}

impl ALUOneOp<u8> for RLA {
    fn execute(value: u8, nf: bool, hf: bool, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            zf: Some(false),
            ..RL::execute(value, nf, hf, cf)
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RRA {}

impl ALUOp for RRA {
    const STR: &'static str = "RRA";
}

impl ALUOneOp<u8> for RRA {
    fn execute(value: u8, nf: bool, hf: bool, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            zf: Some(false),
            ..RR::execute(value, nf, hf, cf)
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DAA {}

impl ALUOp for DAA {
    const STR: &'static str = "DAA";
}

impl ALUOneOp<u8> for DAA {
    fn execute(mut value: u8, nf: bool, hf: bool, cf: bool) -> ALUOpResult<u8> {
        let mut new_carry = false;
        if !nf {
            if cf || value > 0x99 {
                value = value.wrapping_add(0x60);
                new_carry = true;
            }
            if hf || value & 0x0f > 0x09 {
                value = value.wrapping_add(0x06);
            }
        } else if cf {
            value = value.wrapping_add(if hf { 0x9a } else { 0xa0 });
            new_carry = true;
        } else if hf {
            value = value.wrapping_add(0xfa);
        }

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: None,
            hf: Some(false),
            cf: Some(new_carry),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CPL {}

impl ALUOp for CPL {
    const STR: &'static str = "CPL";
}

impl ALUOneOp<u8> for CPL {
    fn execute(value: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        let value = !value;

        ALUOpResult {
            value: Some(value),
            zf: None,
            nf: Some(true),
            hf: Some(true),
            cf: None,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SCF {}

impl ALUOp for SCF {
    const STR: &'static str = "SCF";
}

impl ALUOneOp<u8> for SCF {
    fn execute(_: u8, _: bool, _: bool, _: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            value: None,
            zf: None,
            nf: Some(false),
            hf: Some(false),
            cf: Some(true),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CCF {}

impl ALUOp for CCF {
    const STR: &'static str = "CCF";
}

impl ALUOneOp<u8> for CCF {
    fn execute(_: u8, _: bool, _: bool, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            value: None,
            zf: None,
            nf: Some(false),
            hf: Some(false),
            cf: Some(!cf),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Add {}

impl ALUOp for Add {
    const STR: &'static str = "ADD";
}

impl ALUTwoOp<u8, u8> for Add {
    fn execute(a: u8, b: u8, _: bool) -> ALUOpResult<u8> {
        let (value, new_carry) = a.overflowing_add(b);

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF) + (b & 0xF) > 0xF),
            cf: Some(new_carry),
        }
    }
}

impl ALUTwoOp<u16, u16> for Add {
    fn execute(a: u16, b: u16, _: bool) -> ALUOpResult<u16> {
        let (value, _) = a.overflowing_add(b);

        let hf_mask = (1u16 << 11) | (1u16 << 11).wrapping_sub(1);

        ALUOpResult {
            value: Some(value),
            zf: None,
            nf: Some(false),
            hf: Some((a & hf_mask) + (b & hf_mask) > hf_mask),
            cf: Some(a > 0xFFFF - b),
        }
    }
}

impl ALUTwoOp<u16, i8> for Add {
    fn execute(a: u16, b: i8, _: bool) -> ALUOpResult<u16> {
        let b = b as u16;
        let value = a.wrapping_add(b);

        let hf_mask = (1u16 << 3) | (1u16 << 3).wrapping_sub(1);
        let cf_mask = (1u16 << 7) | (1u16 << 7).wrapping_sub(1);

        ALUOpResult {
            value: Some(value),
            zf: Some(false),
            nf: Some(false),
            hf: Some((a & hf_mask) + (b & hf_mask) > hf_mask),
            cf: Some((a & cf_mask) + (b & cf_mask) > cf_mask),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdC {}

impl ALUOp for AdC {
    const STR: &'static str = "ADC";
}

impl ALUTwoOp<u8, u8> for AdC {
    fn execute(a: u8, b: u8, cf: bool) -> ALUOpResult<u8> {
        let value = a.wrapping_add(b).wrapping_add(cf as u8);

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF) + (b & 0xF) + (cf as u8) > 0xf),
            cf: Some(a as u16 + b as u16 + cf as u16 > 0xff),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sub {}

impl ALUOp for Sub {
    const STR: &'static str = "SUB";
}

impl ALUTwoOp<u8, u8> for Sub {
    fn execute(a: u8, b: u8, _: bool) -> ALUOpResult<u8> {
        let value = a.wrapping_sub(b);

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF) & 0x10 != 0),
            cf: Some(a < b),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SbC {}

impl ALUOp for SbC {
    const STR: &'static str = "SBC";
}

impl ALUTwoOp<u8, u8> for SbC {
    fn execute(a: u8, b: u8, cf: bool) -> ALUOpResult<u8> {
        let value = a.wrapping_sub(b).wrapping_sub(cf as u8);

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF).wrapping_sub(cf as u8) & 0x10 != 0),
            cf: Some((a as u16) < (b as u16) + (cf as u16)),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct And {}

impl ALUOp for And {
    const STR: &'static str = "AND";
}

impl ALUTwoOp<u8, u8> for And {
    fn execute(a: u8, b: u8, _: bool) -> ALUOpResult<u8> {
        let value = a & b;

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(true),
            cf: Some(false),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Xor {}

impl ALUOp for Xor {
    const STR: &'static str = "XOR";
}

impl ALUTwoOp<u8, u8> for Xor {
    fn execute(a: u8, b: u8, _: bool) -> ALUOpResult<u8> {
        let value = a ^ b;

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Or {}

impl ALUOp for Or {
    const STR: &'static str = "OR";
}

impl ALUTwoOp<u8, u8> for Or {
    fn execute(a: u8, b: u8, _: bool) -> ALUOpResult<u8> {
        let value = a | b;

        ALUOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cp {}

impl ALUOp for Cp {
    const STR: &'static str = "CP";
}

impl ALUTwoOp<u8, u8> for Cp {
    fn execute(a: u8, b: u8, cf: bool) -> ALUOpResult<u8> {
        ALUOpResult {
            value: None,
            ..Sub::execute(a, b, cf)
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bit {}

impl ALUOp for Bit {
    const STR: &'static str = "BIT";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Bit {
    fn execute(value: u8) -> ALUOpResult<u8> {
        ALUOpResult {
            value: None,
            zf: Some(((value >> BIT_POS) & 1) == 0),
            nf: Some(false),
            hf: Some(true),
            cf: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Set {}

impl ALUOp for Set {
    const STR: &'static str = "SET";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Set {
    fn execute(value: u8) -> ALUOpResult<u8> {
        ALUOpResult {
            value: Some(value | (1 << BIT_POS)),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Res {}

impl ALUOp for Res {
    const STR: &'static str = "RES";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Res {
    fn execute(value: u8) -> ALUOpResult<u8> {
        ALUOpResult {
            value: Some(value & (!(1 << BIT_POS))),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}
