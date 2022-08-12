use super::frame_sequencer::FrameSequencer;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnvelopeDirection {
    Increase,
    Decrease,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VolumeEnvelope {
    initial: u8,
    current: u8,
    direction: EnvelopeDirection,
    period: u8,
    period_timer: u8,
}

impl VolumeEnvelope {
    pub fn new() -> Self {
        Self {
            initial: 0,
            current: 0,
            direction: EnvelopeDirection::Decrease,
            period: 0,
            period_timer: 0,
        }
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) {
        if !frame_sequencer.should_tick_volume_envelope() {
            return;
        }

        if self.period == 0 {
            return;
        }

        if self.period_timer > 0 {
            self.period_timer -= 1;

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current > 0 && self.direction == EnvelopeDirection::Decrease)
                    || (self.current < 0xF && self.direction == EnvelopeDirection::Increase)
                {
                    match self.direction {
                        EnvelopeDirection::Increase => self.current += 1,
                        EnvelopeDirection::Decrease => self.current -= 1,
                    }
                }
            }
        }
    }

    pub fn trigger(&mut self) {
        self.current = self.initial;
        self.period_timer = self.period;
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn initial(&self) -> u8 {
        self.initial
    }

    pub fn set_initial(&mut self, value: u8) {
        self.initial = value & 0b1111;
    }

    pub fn period(&self) -> u8 {
        self.period
    }

    pub fn set_period(&mut self, value: u8) {
        self.period = value & 0b111;
    }

    pub fn direction(&self) -> EnvelopeDirection {
        self.direction
    }

    pub fn set_direction(&mut self, value: EnvelopeDirection) {
        self.direction = value;
    }
}
