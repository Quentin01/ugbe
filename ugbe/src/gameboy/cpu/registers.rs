use paste::paste;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

macro_rules! define_r8 {
    ($reg:ident) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<$reg:lower>](&self) -> u8 {
                self.[<$reg:lower>]
            }

            #[allow(dead_code)]
            pub fn [<set_ $reg:lower>](&mut self, value: u8) {
                self.[<$reg:lower>] = value;
            }
        }
    };
}

macro_rules! define_r16 {
    ($reg:ident) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<$reg:lower>](&self) -> u16 {
                self.[<$reg:lower>]
            }

            #[allow(dead_code)]
            pub fn [<set_ $reg:lower>](&mut self, value: u16) {
                self.[<$reg:lower>] = value;
            }
        }
    };
}

macro_rules! define_split_r16 {
    ($msb:ident, $lsb:ident) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<$msb:lower $lsb:lower>](&self) -> u16 {
                u16::from_be_bytes([self.[<$msb:lower>](), self.[<$lsb:lower>]()])
            }

            #[allow(dead_code)]
            pub fn [<set_ $msb:lower $lsb:lower>](&mut self, value: u16) {
                let [msb, lsb] = value.to_be_bytes();
                self.[<set_ $msb:lower>](msb);
                self.[<set_ $lsb:lower>](lsb);
            }
        }
    };
}

macro_rules! define_flag {
    ($flag:ident, $pos:literal) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<$flag:lower f>](&self) -> bool {
                ((self.f >> $pos) & 1) != 0
            }

            #[allow(dead_code)]
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
    pub fn new() -> Self {
        // TODO: Put the right default value for registers
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
        }
    }

    define_r8!(A);
    define_r8!(B);
    define_r8!(C);
    define_r8!(D);
    define_r8!(E);
    define_r8!(H);
    define_r8!(L);

    #[allow(dead_code)]
    pub fn f(&self) -> u8 {
        self.f
    }

    #[allow(dead_code)]
    pub fn set_f(&mut self, value: u8) {
        // The last four bits of F can't be set
        self.f = value & 0xF0;
    }

    define_r16!(SP);
    define_r16!(PC);

    define_split_r16!(A, F);
    define_split_r16!(B, C);
    define_split_r16!(D, E);
    define_split_r16!(H, L);

    define_flag!(Z, 7);
    define_flag!(N, 6);
    define_flag!(H, 5);
    define_flag!(C, 4);
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
