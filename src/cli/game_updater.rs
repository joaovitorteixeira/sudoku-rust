use std::sync::{mpsc::Receiver};

pub struct GameUpdater {
    board_rx: Receiver<String>,
}

impl GameUpdater {
    pub fn new(board_rx: Receiver<String>) -> Self {
        GameUpdater { board_rx }
    }

    pub fn listen(self) -> Result<(), String> {
        loop {
            match self.board_rx.recv() {
                Ok(message) => println!("{message}"),
                Err(_) => return Err("Channel is disconnected".into()),
            }
        }
    }
}
