#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameSequencer {
    counter: usize,
    step: u8,

    should_tick_length_counter: bool,
    should_tick_volume_envelope: bool,
    should_tick_frequency_sweep: bool,
}

impl FrameSequencer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            step: 0,
            should_tick_length_counter: false,
            should_tick_volume_envelope: false,
            should_tick_frequency_sweep: false,
        }
    }

    pub fn tick(&mut self) {
        self.counter += 1;

        self.should_tick_length_counter = false;
        self.should_tick_volume_envelope = false;
        self.should_tick_frequency_sweep = false;

        if self.counter % 8192 == 0 {
            self.step = (self.step + 1) % 8;

            self.should_tick_length_counter =
                self.step == 0 || self.step == 2 || self.step == 4 || self.step == 6;
            self.should_tick_volume_envelope = self.step == 7;
            self.should_tick_frequency_sweep = self.step == 2 || self.step == 6;
        }
    }

    pub fn trigger(&mut self) {
        self.counter = 0;
        self.step = 0;
        self.should_tick_length_counter = true;
        self.should_tick_volume_envelope = false;
        self.should_tick_frequency_sweep = false;
    }

    pub fn should_tick_length_counter(&self) -> bool {
        self.should_tick_length_counter
    }

    pub fn should_tick_volume_envelope(&self) -> bool {
        self.should_tick_volume_envelope
    }

    pub fn should_tick_frequency_sweep(&self) -> bool {
        self.should_tick_frequency_sweep
    }
}
