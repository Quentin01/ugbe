pub(super) const FREQUENCY: usize = 4_194_304;
const NANOS_PER_T_CYCLE: f64 = 1_000_000_000f64 / FREQUENCY as f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clock {
    t_cycle_count: usize,
}

impl Clock {
    pub fn new() -> Self {
        Self { t_cycle_count: 0 }
    }

    pub(super) fn tick(&mut self) {
        self.t_cycle_count = self.t_cycle_count.wrapping_add(1);
    }

    pub fn is_m_cycle(&self) -> bool {
        self.t_cycle_count % 4 == 0
    }

    pub fn is_apu_cycle(&self) -> bool {
        self.t_cycle_count % 2 == 0
    }

    pub fn now(&self) -> Instant {
        Instant {
            t_cycle_count: self.t_cycle_count,
            extra_nano_seconds: 0.0,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Instant {
    t_cycle_count: usize,
    extra_nano_seconds: f64,
}

impl Instant {
    pub fn restart(&mut self, clock: &Clock) -> std::time::Duration {
        let elapsed_t_cycles = if clock.t_cycle_count > self.t_cycle_count {
            clock.t_cycle_count - self.t_cycle_count
        } else {
            usize::MAX - self.t_cycle_count + clock.t_cycle_count
        };

        assert!(clock.t_cycle_count >= self.t_cycle_count);

        let nano_seconds_duration =
            NANOS_PER_T_CYCLE * elapsed_t_cycles as f64 + self.extra_nano_seconds;
        let nano_seconds_duration_int = nano_seconds_duration as u32;

        self.extra_nano_seconds = nano_seconds_duration - nano_seconds_duration_int as f64;
        self.t_cycle_count = clock.t_cycle_count;

        std::time::Duration::new(0, nano_seconds_duration_int)
    }
}
