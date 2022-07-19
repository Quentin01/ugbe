use paste::paste;

#[derive(Debug, Default, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    /// Extra register used as temporary between M-cycles
    pub x: u8,
    /// Extra register used as temporary between M-cycles
    pub y: u8,
    pub pc: u16,
    pub sp: u16,
}

macro_rules! define_split_r16 {
    ($msb:ident, $lsb:ident) => {
        paste! {
            pub fn [<$msb:lower $lsb:lower>](&self) -> u16 {
                u16::from_be_bytes([self.[<$msb:lower>], self.[<$lsb:lower>]])
            }

            pub fn [<set_ $msb:lower $lsb:lower>](&mut self, value: u16) {
                [self.[<$msb:lower>], self.[<$lsb:lower>]] = value.to_be_bytes()
            }

            pub fn [<set_msb_ $msb:lower $lsb:lower>](&mut self, value: u8) {
                self.[<$msb:lower>] = value;
            }

            pub fn [<set_lsb_ $msb:lower $lsb:lower>](&mut self, value: u8) {
                self.[<$lsb:lower>] = value;
            }
        }
    };
}

macro_rules! define_flag {
    ($flag:ident, $pos:literal) => {
        paste! {
            pub fn [<$flag:lower f>](&self) -> bool {
                ((self.f >> $pos) & 1) != 0
            }

            pub fn [<set_ $flag:lower f>](&mut self, value: bool) {
                if value {
                    self.f |= 1 << $pos;
                } else {
                    self.f &= !(1 << $pos);
                }
            }
        }
    };
}

impl Registers {
    define_split_r16!(A, F);
    define_split_r16!(B, C);
    define_split_r16!(D, E);
    define_split_r16!(H, L);
    define_split_r16!(X, Y);

    define_flag!(Z, 7);
    define_flag!(N, 6);
    define_flag!(H, 5);
    define_flag!(C, 4);
}
