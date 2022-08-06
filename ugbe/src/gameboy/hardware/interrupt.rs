use super::super::interrupt::{Kind as InterruptKind, Line as InterruptLine};

#[derive(Debug)]
pub struct Interrupt {
    interrupt_enable: u8,
    interrupt_flags: u8,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            interrupt_enable: 0,
            interrupt_flags: 0,
        }
    }

    pub fn enable(&self) -> u8 {
        self.interrupt_enable | 0b11100000
    }

    pub fn set_enable(&mut self, value: u8) {
        self.interrupt_enable = value & 0b11111
    }

    pub fn flags(&self) -> u8 {
        self.interrupt_flags | 0b11100000
    }

    pub fn set_flags(&mut self, value: u8) {
        self.interrupt_flags = value & 0b11111
    }
}

impl InterruptLine for Interrupt {
    fn highest_priority(&self) -> Option<InterruptKind> {
        let interrupt_result = self.interrupt_enable & self.interrupt_flags;
        for interrupt_kind in [
            InterruptKind::VBlank,
            InterruptKind::Stat,
            InterruptKind::Timer,
            InterruptKind::Serial,
            InterruptKind::Joypad,
        ] {
            if interrupt_result & (interrupt_kind as u8) != 0 {
                return Some(interrupt_kind);
            }
        }
        None
    }

    fn ack(&mut self, kind: InterruptKind) {
        self.interrupt_flags &= !(kind as u8);
    }

    fn request(&mut self, kind: InterruptKind) {
        self.interrupt_flags |= kind as u8;
    }

    fn flags_not_empty(&self) -> bool {
        self.interrupt_flags & 0b11111 != 0
    }
}
