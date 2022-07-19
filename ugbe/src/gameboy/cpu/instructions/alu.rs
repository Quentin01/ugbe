pub trait Alu {
    const STR: &'static str;
}

pub struct Alu8Result {
    pub value: Option<u8>,
    pub zf: Option<bool>,
    pub nf: Option<bool>,
    pub hf: Option<bool>,
    pub cf: Option<bool>,
}

pub trait Alu8: Alu {
    fn execute8(a: u8, b: u8, cf: bool) -> Alu8Result;
}

pub trait Alu16: Alu {
    fn execute16(a: u16, b: u16) -> u16;
}

pub struct Add {}

impl Alu for Add {
    const STR: &'static str = "ADD";
}

impl Alu8 for Add {
    fn execute8(a: u8, b: u8, _: bool) -> Alu8Result {
        let (value, new_carry) = a.overflowing_add(b);

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF).checked_add(b | 0xF).is_none()),
            cf: Some(new_carry),
        }
    }
}

impl Alu16 for Add {
    fn execute16(a: u16, b: u16) -> u16 {
        todo!("Execute ADD 16")
    }
}

pub struct Adc {}

impl Alu for Adc {
    const STR: &'static str = "ADC";
}

impl Alu8 for Adc {
    fn execute8(a: u8, b: u8, cf: bool) -> Alu8Result {
        let value = a.wrapping_add(b).wrapping_add(cf as u8);

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some((a & 0xF) + (b & 0xF) + (cf as u8) > 0xf),
            cf: Some(a as u16 + b as u16 + cf as u16 > 0xff),
        }
    }
}

pub struct Sub {}

impl Alu for Sub {
    const STR: &'static str = "SUB";
}

impl Alu8 for Sub {
    fn execute8(a: u8, b: u8, _: bool) -> Alu8Result {
        let value = a.wrapping_sub(b);

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF) & 0x10 != 0),
            cf: Some(a < b),
        }
    }
}

pub struct Sbc {}

impl Alu for Sbc {
    const STR: &'static str = "SBC";
}

impl Alu8 for Sbc {
    fn execute8(a: u8, b: u8, cf: bool) -> Alu8Result {
        let value = a.wrapping_sub(b).wrapping_sub(cf as u8);

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(true),
            hf: Some((a & 0xF).wrapping_sub(b & 0xF) & 0x10 != 0),
            cf: Some((a as u16) < (b as u16) + (cf as u16)),
        }
    }
}

pub struct And {}

impl Alu for And {
    const STR: &'static str = "AND";
}

impl Alu8 for And {
    fn execute8(a: u8, b: u8, _: bool) -> Alu8Result {
        let value = a & b;

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(true),
            cf: Some(false),
        }
    }
}

pub struct Xor {}

impl Alu for Xor {
    const STR: &'static str = "XOR";
}

impl Alu8 for Xor {
    fn execute8(a: u8, b: u8, _: bool) -> Alu8Result {
        let value = a ^ b;

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

pub struct Or {}

impl Alu for Or {
    const STR: &'static str = "OR";
}

impl Alu8 for Or {
    fn execute8(a: u8, b: u8, _: bool) -> Alu8Result {
        let value = a & b;

        Alu8Result {
            value: Some(value),
            zf: Some(value == 0),
            nf: Some(false),
            hf: Some(false),
            cf: Some(false),
        }
    }
}

pub struct Cp {}

impl Alu for Cp {
    const STR: &'static str = "CP";
}

impl Alu8 for Cp {
    fn execute8(a: u8, b: u8, cf: bool) -> Alu8Result {
        Alu8Result {
            value: None,
            ..Sub::execute8(a, b, cf)
        }
    }
}

pub struct Bit {}

impl Alu for Bit {
    const STR: &'static str = "BIT";
}

impl Alu8 for Bit {
    fn execute8(value: u8, pos: u8, _: bool) -> Alu8Result {
        Alu8Result {
            value: None,
            zf: Some(((value >> pos) & 1) == 0),
            nf: Some(false),
            hf: Some(true),
            cf: None,
        }
    }
}

pub struct Set {}

impl Alu for Set {
    const STR: &'static str = "SET";
}

impl Alu8 for Set {
    fn execute8(value: u8, pos: u8, _: bool) -> Alu8Result {
        Alu8Result {
            value: Some(value | (1 << pos)),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}

pub struct Res {}

impl Alu for Res {
    const STR: &'static str = "RES";
}

impl Alu8 for Res {
    fn execute8(value: u8, pos: u8, _: bool) -> Alu8Result {
        Alu8Result {
            value: Some(value & (!(1 << pos))),
            zf: None,
            nf: None,
            hf: None,
            cf: None,
        }
    }
}
