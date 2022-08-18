use super::frame_sequencer::FrameSequencer;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LengthCounter<const NB_BITS: u8> {
    enabled: bool,
    counter: u16,
}

impl<const NB_BITS: u8> LengthCounter<NB_BITS> {
    const MAX_LENGTH: u16 = 1 << NB_BITS;
    const VALUE_MASK: u8 = (Self::MAX_LENGTH - 1) as u8;

    pub fn new() -> Self {
        Self {
            enabled: false,
            counter: Self::MAX_LENGTH,
        }
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) {
        if !frame_sequencer.should_tick_length_counter() || !self.enabled {
            return;
        }

        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    pub fn trigger(&mut self, frame_sequencer: &FrameSequencer) {
        if self.counter == 0 {
            self.counter = Self::MAX_LENGTH;

            if self.enabled && frame_sequencer.should_have_tick_length_counter() {
                self.counter -= 1;
            }
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self, enabled: bool, frame_sequencer: &FrameSequencer) {
        let was_enabled = self.enabled;
        self.enabled = enabled;

        if !was_enabled
            && self.enabled
            && frame_sequencer.should_have_tick_length_counter()
            && self.counter > 0
        {
            self.counter -= 1;
        }
    }

    pub fn value(&self) -> u16 {
        self.counter
    }

    pub fn set_value(&mut self, value: u8) {
        self.counter = Self::MAX_LENGTH - (value & Self::VALUE_MASK) as u16;
    }
}
