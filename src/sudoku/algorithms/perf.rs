use std::time::{Duration, Instant};

pub struct PerfTracker {
    actions: u64,
    start: Option<Instant>,
    end: Option<Instant>,
}

impl PerfTracker {
    pub fn new() -> Self {
        Self {
            actions: 0,
            start: None,
            end: None,
        }
    }

    pub fn start(&mut self) {
        self.actions = 0;
        self.start = Some(Instant::now());
        self.end = None;
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
                Some(end) => Some(end.duration_since(start)),
                None => Some(Instant::now().duration_since(start)),
            },
        }
    }

    pub fn print_summary(&self) {
        let elapsed_str = match self.elapsed() {
            Some(d) => format!("{:.3}s", d.as_secs_f64()),
            None => "not started".to_string(),
        };

        eprintln!("Perf: actions={} elapsed={}", self.actions, elapsed_str);
    }
}
