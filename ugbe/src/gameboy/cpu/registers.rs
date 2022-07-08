const Z_FLAG_BIT: u8 = 7;
const N_FLAG_BIT: u8 = 6;
const H_FLAG_BIT: u8 = 5;
const C_FLAG_BIT: u8 = 4;

#[derive(Debug, Copy, Clone, Default)]
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

#[derive(Debug, Copy, Clone)]
pub enum R16ToR8 {
    High,
    Low,
}

#[derive(Debug, Copy, Clone)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    X,
    Y,
    BC(R16ToR8),
    DE(R16ToR8),
    HL(R16ToR8),
    XY(R16ToR8),
    PC(R16ToR8),
    SP(R16ToR8),
}

#[derive(Debug, Copy, Clone)]
pub enum R16 {
    AF,
    BC,
    DE,
    HL,
    XY,
    SP,
    PC,
}

#[derive(Debug, Copy, Clone)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

impl Registers {
    pub fn read_byte(&self, register: R8) -> u8 {
        match register {
            R8::A => self.a,
            R8::B => self.b,
            R8::C => self.c,
            R8::D => self.d,
            R8::E => self.e,
            R8::F => self.f,
            R8::H => self.h,
            R8::L => self.l,
            R8::X => self.x,
            R8::Y => self.y,
            R8::BC(byte) => match byte {
                R16ToR8::High => self.b,
                R16ToR8::Low => self.c,
            },
            R8::DE(byte) => match byte {
                R16ToR8::High => self.d,
                R16ToR8::Low => self.e,
            },
            R8::HL(byte) => match byte {
                R16ToR8::High => self.h,
                R16ToR8::Low => self.l,
            },
            R8::XY(byte) => match byte {
                R16ToR8::High => self.x,
                R16ToR8::Low => self.y,
            },
            R8::PC(byte) => match byte {
                R16ToR8::High => self.pc.to_be_bytes()[0],
                R16ToR8::Low => self.pc.to_be_bytes()[1],
            },
            R8::SP(byte) => match byte {
                R16ToR8::High => self.sp.to_be_bytes()[0],
                R16ToR8::Low => self.sp.to_be_bytes()[1],
            },
        }
    }

    pub fn write_byte(&mut self, register: R8, value: u8) {
        match register {
            R8::A => self.a = value,
            R8::B => self.b = value,
            R8::C => self.c = value,
            R8::D => self.d = value,
            R8::E => self.e = value,
            R8::F => self.f = value,
            R8::H => self.h = value,
            R8::L => self.l = value,
            R8::X => self.x = value,
            R8::Y => self.y = value,
            R8::BC(byte) => match byte {
                R16ToR8::High => self.b = value,
                R16ToR8::Low => self.c = value,
            },
            R8::DE(byte) => match byte {
                R16ToR8::High => self.d = value,
                R16ToR8::Low => self.e = value,
            },
            R8::HL(byte) => match byte {
                R16ToR8::High => self.h = value,
                R16ToR8::Low => self.l = value,
            },
            R8::XY(byte) => match byte {
                R16ToR8::High => self.x = value,
                R16ToR8::Low => self.y = value,
            },
            R8::PC(byte) => match byte {
                R16ToR8::High => self.pc = u16::from_be_bytes([value, self.pc.to_be_bytes()[1]]),
                R16ToR8::Low => self.pc = u16::from_be_bytes([self.pc.to_be_bytes()[0], value]),
            },
            R8::SP(byte) => match byte {
                R16ToR8::High => self.sp = u16::from_be_bytes([value, self.sp.to_be_bytes()[1]]),
                R16ToR8::Low => self.sp = u16::from_be_bytes([self.sp.to_be_bytes()[0], value]),
            },
        }
    }

    pub fn read_word(&self, register: R16) -> u16 {
        match register {
            R16::AF => u16::from_be_bytes([self.a, self.f]),
            R16::BC => u16::from_be_bytes([self.b, self.c]),
            R16::DE => u16::from_be_bytes([self.d, self.e]),
            R16::HL => u16::from_be_bytes([self.h, self.l]),
            R16::XY => u16::from_be_bytes([self.x, self.y]),
            R16::SP => self.sp,
            R16::PC => self.pc,
        }
    }

    pub fn write_word(&mut self, register: R16, value: u16) {
        match register {
            R16::AF => [self.a, self.f] = value.to_be_bytes(),
            R16::BC => [self.b, self.c] = value.to_be_bytes(),
            R16::DE => [self.d, self.e] = value.to_be_bytes(),
            R16::HL => [self.h, self.l] = value.to_be_bytes(),
            R16::XY => [self.x, self.y] = value.to_be_bytes(),
            R16::SP => self.sp = value,
            R16::PC => self.pc = value,
        }
    }

    pub fn read_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => ((self.f >> Z_FLAG_BIT) & 1) != 0,
            Flag::N => ((self.f >> N_FLAG_BIT) & 1) != 0,
            Flag::H => ((self.f >> H_FLAG_BIT) & 1) != 0,
            Flag::C => ((self.f >> C_FLAG_BIT) & 1) != 0,
        }
    }

    pub fn write_flag(&mut self, flag: Flag, value: bool) {
        if value {
            match flag {
                Flag::Z => self.f |= 1 << Z_FLAG_BIT,
                Flag::N => self.f |= 1 << N_FLAG_BIT,
                Flag::H => self.f |= 1 << H_FLAG_BIT,
                Flag::C => self.f |= 1 << C_FLAG_BIT,
            }
        } else {
            match flag {
                Flag::Z => self.f &= !(1 << Z_FLAG_BIT),
                Flag::N => self.f &= !(1 << N_FLAG_BIT),
                Flag::H => self.f &= !(1 << H_FLAG_BIT),
                Flag::C => self.f &= !(1 << C_FLAG_BIT),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Flag, R16ToR8, Registers, R8};

    #[test]
    fn read_pc_low_and_high() {
        let registers = Registers {
            pc: 0xAABB,
            ..Registers::default()
        };

        assert_eq!(registers.read_byte(R8::PC(R16ToR8::Low)), 0xBB);
        assert_eq!(registers.read_byte(R8::PC(R16ToR8::High)), 0xAA);
    }

    #[test]
    fn use_flags() {
        let mut registers = Registers::default();

        assert!(!registers.read_flag(Flag::Z));

        registers.write_flag(Flag::Z, true);
        assert_eq!(registers.f, 0x80);
        assert!(registers.read_flag(Flag::Z));

        registers.write_flag(Flag::Z, false);
        assert_eq!(registers.f, 0x00);
        assert!(!registers.read_flag(Flag::Z));
    }
}
