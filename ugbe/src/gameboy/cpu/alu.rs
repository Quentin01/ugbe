#[derive(Debug, Copy, Clone)]
pub enum Operation8 {
    Xor(super::In8, super::In8),
    Bit(usize, super::In8),
}

impl Operation8 {
    pub fn execute(&self, cpu_context: &mut super::CpuContext) -> u8 {
        match self {
            Self::Xor(a, b) => {
                let value = a.read_byte(cpu_context) ^ b.read_byte(cpu_context);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, value == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, false);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::H, false);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::C, false);
                value
            }
            Self::Bit(bit, a) => {
                let value = a.read_byte(cpu_context);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, ((value >> bit) & 1) == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, false);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::H, true);
                value
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Operation16 {
    AddWithI8(super::In16, super::In8),
}

impl Operation16 {
    pub fn execute(&self, cpu_context: &super::CpuContext) -> u16 {
        match self {
            Self::AddWithI8(a, offset) => {
                (a.read_word(cpu_context) as i32 + offset.read_byte(cpu_context) as i8 as i32)
                    as u16
            }
        }
    }
}
