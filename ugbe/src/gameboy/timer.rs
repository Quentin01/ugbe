use super::components::{InterruptKind, InterruptLine};

#[derive(Debug)]
struct TAC(u8);

impl From<u8> for TAC {
    fn from(value: u8) -> Self {
        Self(value | 0b11111000)
    }
}

impl From<&TAC> for u8 {
    fn from(value: &TAC) -> Self {
        value.0 | 0b11111000
    }
}

impl TAC {
    fn tima_enabled(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    fn counter_bit_pos(&self) -> u8 {
        match self.0 & 0b11 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Timer {
    internal_counter: u16,
    control: TAC,
    modulo: u8,
    counter: u8,
    old_tima_bit: bool,
    overflow_cycles: u8,
    written_during_overflow: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Self {
            internal_counter: 0,
            control: 0.into(),
            modulo: 0,
            counter: 0,
            old_tima_bit: false,
            overflow_cycles: 0,
            written_during_overflow: false,
        }
    }

    pub fn tick(&mut self, interrupt_line: &mut dyn InterruptLine) {
        self.internal_counter = self.internal_counter.wrapping_add(1);

        if self.overflow_cycles > 0 {
            self.overflow_cycles -= 1;

            if self.overflow_cycles == 0 && !self.written_during_overflow {
                self.counter = self.modulo;
                interrupt_line.request(InterruptKind::Timer);
            }
        } else {
            let internal_counter_bit =
                ((self.internal_counter >> self.control.counter_bit_pos()) & 0b1) == 1;
            let tima_enabled = self.control.tima_enabled();
            let tima_bit = tima_enabled && internal_counter_bit;

            if self.old_tima_bit && !tima_bit {
                self.counter = self.counter.wrapping_add(1);

                if self.counter == 0 {
                    self.overflow_cycles = 4;
                    self.written_during_overflow = false;
                }
            }

            self.old_tima_bit = tima_bit;
        }
    }

    pub fn read_div(&self) -> u8 {
        let [msb, _] = self.internal_counter.to_be_bytes();
        msb
    }

    pub fn write_div(&mut self, _: u8) {
        self.internal_counter = 0;
    }

    pub fn read_tima(&self) -> u8 {
        self.counter
    }

    pub fn write_tima(&mut self, value: u8) {
        self.counter = value;
        self.written_during_overflow = true;
    }

    pub fn read_tma(&self) -> u8 {
        self.modulo
    }

    pub fn write_tma(&mut self, value: u8) {
        self.modulo = value;
    }

    pub fn read_tac(&self) -> u8 {
        (&self.control).into()
    }

    pub fn write_tac(&mut self, value: u8) {
        self.control = value.into()
    }
}
