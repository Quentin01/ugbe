use super::hardware::Hardware;

#[derive(Debug, Clone)]
struct Registers {
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

impl Registers {
    fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, value: u16) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct CPU {
    registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers {
                // TODO: Put the right default value for registers
                a: 0x0,
                b: 0x0,
                c: 0x0,
                d: 0x0,
                e: 0x0,
                f: 0x0,
                h: 0x0,
                l: 0x0,
                pc: 0x0,
                sp: 0x0,
            },
        }
    }

    pub fn tick(&mut self, hardware: &mut Hardware) {
        // TODO: Implement correctly instructions
        let opcode = hardware.read_byte(self.registers.pc);
        self.registers.pc += 1;

        match opcode {
            0x21 => {
                // LD HL, u16
                let lsb = hardware.read_byte(self.registers.pc);
                self.registers.pc += 1;

                let msb = hardware.read_byte(self.registers.pc);
                self.registers.pc += 1;

                let immediate = (msb as u16) << 8 | lsb as u16;
                println!("LD HL, ${:x}", immediate);

                self.registers.h = msb;
                self.registers.l = lsb;
            }
            0x31 => {
                // LD SP, u16
                let lsb = hardware.read_byte(self.registers.pc);
                self.registers.pc += 1;

                let msb = hardware.read_byte(self.registers.pc);
                self.registers.pc += 1;

                let immediate = (msb as u16) << 8 | lsb as u16;

                println!("LD SP, ${:x}", immediate);
                self.registers.sp = immediate;
            }
            0xAF => {
                // XOR A
                self.registers.a ^= self.registers.a;

                println!("XOR A");
            }
            _ => {
                panic!("Unknown opcode: {:x} ({:?})", opcode, self);
            }
        }
    }
}
