#[derive(Debug, Copy, Clone)]
pub enum Operation8 {
    Add(super::In8, super::In8),
    Sub(super::In8, super::In8),
    Xor(super::In8, super::In8),
    Cp(super::In8, super::In8),
    Bit(usize, super::In8),
    Inc(super::In8),
    Dec(super::In8),
    Rl(super::In8),
    RlA,
}

impl Operation8 {
    pub fn execute(&self, cpu_context: &mut super::CpuContext) -> u8 {
        match self {
            Self::Add(a, b) => {
                let old_value_a = a.read_byte(cpu_context);
                let old_value_b = b.read_byte(cpu_context);
                let (value, new_carry) = old_value_a.overflowing_add(old_value_b);

                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, value == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, false);
                cpu_context.registers.write_flag(
                    super::registers::Flag::H,
                    (old_value_a & 0xF).checked_add(old_value_b | 0xF).is_none(),
                );
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::C, new_carry);

                value
            }
            Self::Sub(a, b) => {
                let old_value_a = a.read_byte(cpu_context);
                let old_value_b = b.read_byte(cpu_context);
                let value = old_value_a.wrapping_sub(old_value_b);

                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, value == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, true);
                cpu_context.registers.write_flag(
                    super::registers::Flag::H,
                    (old_value_a & 0xF).wrapping_sub(old_value_b & 0xF) & 0x10 != 0,
                );
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::C, old_value_a < old_value_b);

                value
            }
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
            Self::Cp(a, b) => {
                Self::Sub(*a, *b).execute(cpu_context);
                0
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

                0
            }
            Self::Inc(a) => {
                let old_value = a.read_byte(cpu_context);
                let value = old_value.wrapping_add(1);

                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, value == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, false);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::H, old_value & 0xF == 0xF);

                value
            }
            Self::Dec(a) => {
                let old_value = a.read_byte(cpu_context);
                let value = old_value.wrapping_sub(1);

                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, value == 0);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::N, false);
                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::H, old_value & 0xF == 0x0);

                value
            }
            Self::Rl(a) => {
                let cf = cpu_context.registers.read_flag(super::registers::Flag::C) as u8;
                let old_value = a.read_byte(cpu_context);
                let value = (old_value << 1) | cf;

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
                    .write_flag(super::registers::Flag::C, old_value & 0x80 != 0);

                value
            }
            Self::RlA => {
                let value = Self::Rl(super::In8::R8(super::registers::R8::A)).execute(cpu_context);

                cpu_context
                    .registers
                    .write_flag(super::registers::Flag::Z, false);

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
                ((a.read_word(cpu_context) as i32) + ((offset.read_byte(cpu_context) as i8) as i32))
                    as u16
            }
        }
    }
}
