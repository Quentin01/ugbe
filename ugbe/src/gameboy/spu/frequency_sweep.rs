use super::frame_sequencer::FrameSequencer;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FrequencyDirection {
    Increase,
    Decrease,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrequencySweep {
    enabled: bool,
    current: u16,
    shadow: u16,
    did_decrease: bool,
    direction: FrequencyDirection,
    period: u8,
    period_timer: u8,
    shift: u8,
}

impl FrequencySweep {
    pub fn new() -> Self {
        Self {
            enabled: false,
            current: 0,
            shadow: 0,
            did_decrease: false,
            direction: FrequencyDirection::Increase,
            period: 0,
            period_timer: 0,
            shift: 0,
        }
    }

    fn calculate_frequency(&mut self) -> (u16, bool) {
        let offset = self.shadow as usize >> self.shift;

        let new = match self.direction {
            FrequencyDirection::Increase => (self.shadow as usize).wrapping_add(offset),
            FrequencyDirection::Decrease => {
                self.did_decrease = true;
                (self.shadow as usize).wrapping_sub(offset)
            }
        };

        (new as u16, new <= 2047)
    }

    pub fn tick(&mut self, frame_sequencer: &FrameSequencer) -> bool {
        if !frame_sequencer.should_tick_frequency_sweep() {
            return true;
        }

        if self.period_timer > 0 {
            self.period_timer -= 1;

            if self.period_timer == 0 {
                self.period_timer = if self.period == 0 { 8 } else { self.period };

                if self.enabled && self.period > 0 {
                    let (new, keep_enable) = self.calculate_frequency();

                    if new <= 2047 {
                        if self.shift > 0 {
                            self.current = new;
                            self.shadow = new;
                        }

                        // For overflow check
                        let (_, overflow_keep_enable) = self.calculate_frequency();
                        return overflow_keep_enable && keep_enable;
                    }

                    return keep_enable;
                }
            }
        }

        true
    }

    pub fn trigger(&mut self) -> bool {
        self.shadow = self.current;
        self.period_timer = if self.period == 0 { 8 } else { self.period };
        self.enabled = self.period != 0 || self.shift != 0;

        if self.shift != 0 {
            // For overflow check
            let (_, keep_enable) = self.calculate_frequency();
            return keep_enable;
        }

        true
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.direction = FrequencyDirection::Increase;
        self.current = 0;
        self.shadow = 0;
        self.period = 0;
        self.period_timer = 0;
        self.shift = 0;
    }

    pub fn current(&self) -> u16 {
        self.current
    }

    pub fn set_current(&mut self, value: u16) {
        self.current = value & 0b111_1111_1111;
    }

    pub fn period(&self) -> u8 {
        self.period
    }

    pub fn set_period(&mut self, value: u8) {
        self.period = value & 0b111;
    }

    pub fn shift(&self) -> u8 {
        self.shift
    }

    pub fn set_shift(&mut self, value: u8) {
        self.shift = value & 0b111;
    }

    pub fn did_decrease(&self) -> bool {
        self.did_decrease
    }

    pub fn reset_did_decrease(&mut self) {
        self.did_decrease = false;
    }

    pub fn direction(&self) -> FrequencyDirection {
        self.direction
    }

    pub fn set_direction(&mut self, value: FrequencyDirection) {
        self.direction = value;
    }
}
