use std::{
    ops::Sub,
    thread::sleep,
    time::{Duration, Instant},
};

pub struct PerfTracker {
    actions: u64,
    start: Option<Instant>,
    end: Option<Instant>,
    total_sleep_time: u64,
}

impl PerfTracker {
    pub fn new() -> Self {
        Self {
            actions: 0,
            start: None,
            end: None,
            total_sleep_time: 0,
        }
    }

    pub fn start(&mut self) {
        self.actions = 0;
        self.start = Some(Instant::now());
        self.end = None;
    }

    pub fn sleep(&mut self, ms: Option<u64>) {
        match ms {
            Some(ms) => {
                sleep(Duration::from_millis(ms));
                self.total_sleep_time += ms;
            }
            None => return,
        }
    }

    pub fn incr(&mut self) {
        self.actions = self.actions.saturating_add(1);
    }

    pub fn finish(&mut self) {
        self.end = Some(Instant::now());
    }

    pub fn elapsed(&self) -> Option<Duration> {
        match self.start {
            None => None,
            Some(start) => match self.end {
                Some(end) => Some(
                    end.duration_since(start)
                        .sub(Duration::from_millis(self.total_sleep_time)),
                ),
                None => Some(
                    Instant::now()
                        .duration_since(start)
                        .sub(Duration::from_millis(self.total_sleep_time)),
                ),
            },
        }
    }

    pub fn print_summary(&self) {
        let elapsed_str = match self.elapsed() {
            Some(d) => format!("{:.6}s", d.as_secs_f64()),
            None => "not started".to_string(),
        };

        eprintln!("Perf: actions={} elapsed={}", self.actions, elapsed_str);
    }
}
