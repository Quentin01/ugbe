pub trait AluOp {
    const STR: &'static str;
}

pub struct AluOpResult<Value> {
    pub value: Option<Value>,
    pub zf: Option<bool>,
    pub nf: Option<bool>,
    pub hf: Option<bool>,
    pub cf: Option<bool>,
}

pub trait AluOneOp<Value>: AluOp {
    fn execute(value: Value, cf: bool) -> AluOpResult<Value>;
}

pub trait AluTwoOp<DstValue, SrcValue>: AluOp {
    fn execute(a: DstValue, b: SrcValue, cf: bool) -> AluOpResult<DstValue>;
}

pub trait AluBitOp<const BIT_POS: u8>: AluOp {
    fn execute(value: u8) -> AluOpResult<u8>;
}

pub struct Inc {}

impl AluOp for Inc {
    const STR: &'static str = "INC";
}

impl AluOneOp<u8> for Inc {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value.wrapping_add(1);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(value & 0xF == 0xF),
            cf: None,
        }
    }
}

impl AluOneOp<u16> for Inc {
    fn execute(value: u16, _: bool) -> AluOpResult<u16> {
        let new_value = value.wrapping_add(1);

        AluOpResult {
            value: Some(new_value),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

pub struct Dec {}

impl AluOp for Dec {
    const STR: &'static str = "DEC";
}

impl AluOneOp<u8> for Dec {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value.wrapping_sub(1);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(value & 0xF == 0x0),
            cf: None,
        }
    }
}

impl AluOneOp<u16> for Dec {
    fn execute(value: u16, _: bool) -> AluOpResult<u16> {
        let new_value = value.wrapping_sub(1);

        AluOpResult {
            value: Some(new_value),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

pub struct Rlc {}

impl AluOp for Rlc {
    const STR: &'static str = "RLC";
}

impl AluOneOp<u8> for Rlc {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value.rotate_left(1);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

pub struct Rl {}

impl AluOp for Rl {
    const STR: &'static str = "RL";
}

impl AluOneOp<u8> for Rl {
    fn execute(value: u8, cf: bool) -> AluOpResult<u8> {
        let new_value = (value << 1) | (cf as u8);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

pub struct Rrc {}

impl AluOp for Rrc {
    const STR: &'static str = "RRC";
}

impl AluOneOp<u8> for Rrc {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value.rotate_right(1);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

pub struct Rr {}

impl AluOp for Rr {
    const STR: &'static str = "RR";
}

impl AluOneOp<u8> for Rr {
    fn execute(value: u8, cf: bool) -> AluOpResult<u8> {
        let new_value = (value >> 1) | ((cf as u8) << 7);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

pub struct Sla {}

impl AluOp for Sla {
    const STR: &'static str = "SLA";
}

impl AluOneOp<u8> for Sla {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value << 1;

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x80 != 0),
        }
    }
}

pub struct Sra {}

impl AluOp for Sra {
    const STR: &'static str = "SRA";
}

impl AluOneOp<u8> for Sra {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value >> 1 | (value & 0x80);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 != 0),
        }
    }
}

pub struct Swap {}

impl AluOp for Swap {
    const STR: &'static str = "SWAP";
}

impl AluOneOp<u8> for Swap {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = (value >> 4) | (value << 4);

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

pub struct Srl {}

impl AluOp for Srl {
    const STR: &'static str = "SRL";
}

impl AluOneOp<u8> for Srl {
    fn execute(value: u8, _: bool) -> AluOpResult<u8> {
        let new_value = value >> 1;

        AluOpResult {
            value: Some(new_value),
            zf: Some(new_value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(value & 0x1 == 0),
        }
    }
}

pub struct RlA {}

impl AluOp for RlA {
    const STR: &'static str = "RLA";
}

impl AluOneOp<u8> for RlA {
    fn execute(value: u8, cf: bool) -> AluOpResult<u8> {
        AluOpResult {
            zf: Some(false),
            ..Rl::execute(value, cf)
        }
    }
}

pub struct Add {}

impl AluOp for Add {
    const STR: &'static str = "ADD";
}

impl AluTwoOp<u8, u8> for Add {
    fn execute(a: u8, b: u8, _: bool) -> AluOpResult<u8> {
        let (value, new_carry) = a.overflowing_add(b);

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF).checked_add(b | 0xF).is_none()),
            cf: Some(new_carry),
        }
    }
}

pub struct Adc {}

impl AluOp for Adc {
    const STR: &'static str = "ADC";
}

impl AluTwoOp<u8, u8> for Adc {
    fn execute(a: u8, b: u8, cf: bool) -> AluOpResult<u8> {
        let value = a.wrapping_add(b).wrapping_add(cf as u8);

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF) + (b & 0xF) + (cf as u8) > 0xf),
            cf: Some(a as u16 + b as u16 + cf as u16 > 0xff),
        }
    }
}

pub struct Sub {}

impl AluOp for Sub {
    const STR: &'static str = "SUB";
}

impl AluTwoOp<u8, u8> for Sub {
    fn execute(a: u8, b: u8, _: bool) -> AluOpResult<u8> {
        let value = a.wrapping_sub(b);

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF) & 0x10 != 0),
            cf: Some(a < b),
        }
    }
}

pub struct Sbc {}

impl AluOp for Sbc {
    const STR: &'static str = "SBC";
}

impl AluTwoOp<u8, u8> for Sbc {
    fn execute(a: u8, b: u8, cf: bool) -> AluOpResult<u8> {
        let value = a.wrapping_sub(b).wrapping_sub(cf as u8);

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF) & 0x10 != 0),
            cf: Some((a as u16) < (b as u16) + (cf as u16)),
        }
    }
}

pub struct And {}

impl AluOp for And {
    const STR: &'static str = "AND";
}

impl AluTwoOp<u8, u8> for And {
    fn execute(a: u8, b: u8, _: bool) -> AluOpResult<u8> {
        let value = a & b;

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(true),
            cf: Some(false),
        }
    }
}

pub struct Xor {}

impl AluOp for Xor {
    const STR: &'static str = "XOR";
}

impl AluTwoOp<u8, u8> for Xor {
    fn execute(a: u8, b: u8, _: bool) -> AluOpResult<u8> {
        let value = a ^ b;

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

pub struct Or {}

impl AluOp for Or {
    const STR: &'static str = "OR";
}

impl AluTwoOp<u8, u8> for Or {
    fn execute(a: u8, b: u8, _: bool) -> AluOpResult<u8> {
        let value = a & b;

        AluOpResult {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

pub struct Cp {}

impl AluOp for Cp {
    const STR: &'static str = "CP";
}

impl AluTwoOp<u8, u8> for Cp {
    fn execute(a: u8, b: u8, cf: bool) -> AluOpResult<u8> {
        AluOpResult {
            value: None,
            ..Sub::execute(a, b, cf)
        }
    }
}

pub struct Bit {}

impl AluOp for Bit {
    const STR: &'static str = "BIT";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Bit {
    fn execute(value: u8) -> AluOpResult<u8> {
        AluOpResult {
            value: None,
            zf: Some(((value >> BIT_POS) & 1) == 0),
            nf: Some(false),
            hf: Some(true),
            cf: None,
        }
    }
}

pub struct Set {}

impl AluOp for Set {
    const STR: &'static str = "SET";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Set {
    fn execute(value: u8) -> AluOpResult<u8> {
        AluOpResult {
            value: Some(value | (1 << BIT_POS)),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

pub struct Res {}

impl AluOp for Res {
    const STR: &'static str = "RES";
}

impl<const BIT_POS: u8> AluBitOp<BIT_POS> for Res {
    fn execute(value: u8) -> AluOpResult<u8> {
        AluOpResult {
            value: Some(value & (!(1 << BIT_POS))),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}
