use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

pub struct GameUpdater {
    board_rx: Receiver<String>,
    throttle_enabled: bool,
    throttle_ms: u64,
}

impl GameUpdater {
    pub fn new(
        board_rx: Receiver<String>,
        throttle_enabled: Option<bool>,
        throttle_ms: Option<u64>,
    ) -> Self {
        GameUpdater {
            board_rx,
            throttle_enabled: throttle_enabled.or(Some(false)).unwrap(),
            throttle_ms: throttle_ms.or(Some(100)).unwrap(),
        }
    }

    fn print(message: String) -> String {
        print!("{}[2J", 27 as char);
        println!("{}", message);
        message
    }

    pub fn listen(self) -> Result<(), String> {
        if !self.throttle_enabled {
            for message in self.board_rx {
                Self::print(message.clone());
            }
            return Ok(());
        }

        let mut last_print = Instant::now() - Duration::from_millis(self.throttle_ms);
        let interval = Duration::from_millis(self.throttle_ms);
        let mut last_message: Option<String> = None;

        for mut message in self.board_rx {
            let now = Instant::now();

            if now.duration_since(last_print) >= interval {
                message = Self::print(message);
                last_print = now;
            }

            last_message = Some(message.clone());
        }

        if let Some(message) = last_message {
            Self::print(message);
        }

        Ok(())
    }
}
