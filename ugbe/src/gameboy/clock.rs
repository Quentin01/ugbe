pub const FREQUENCY: usize = 4_194_304;
const T_CYCLE_DURATION: std::time::Duration =
    std::time::Duration::new(0, (1_000_000_000f64 / FREQUENCY as f64) as u32);

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
        }
    }
}

#[derive(Clone, Copy)]
pub struct Instant {
    t_cycle_count: usize,
}

impl Instant {
    pub fn elapsed(&self, clock: &Clock) -> std::time::Duration {
        // FIXME: We could have already wrap
        assert!(clock.t_cycle_count >= self.t_cycle_count);
        T_CYCLE_DURATION * (clock.t_cycle_count - self.t_cycle_count) as u32
    }
}
