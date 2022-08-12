use super::frame_sequencer::FrameSequencer;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LengthCounter<const NB_BITS: u8> {
    counter: u16,
    value: u8,
}

impl<const NB_BITS: u8> LengthCounter<NB_BITS> {
    const MAX_LENGTH: u16 = 1 << (NB_BITS + 1);
    const VALUE_MASK: u8 = (Self::MAX_LENGTH - 1) as u8;

    pub fn new() -> Self {
        Self {
            counter: Self::MAX_LENGTH,
            value: 0,
        }
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) {
        if !frame_sequencer.should_tick_length_counter() {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    pub fn trigger(&mut self) {
        self.counter = Self::MAX_LENGTH - self.value as u16;
    }

    pub fn value(&self) -> u16 {
        self.counter
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value & Self::VALUE_MASK;
    }
}
