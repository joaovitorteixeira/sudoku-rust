use std::sync::mpsc::Receiver;

pub struct GameUpdater {
    board_rx: Receiver<String>,
}

impl GameUpdater {
    pub fn new(board_rx: Receiver<String>) -> Self {
        GameUpdater { board_rx }
    }

    pub fn listen(self) -> Result<(), String> {
        for message in self.board_rx {
            print!("{}[2J", 27 as char);
            println!("{message}");
        }

        Ok(())
    }
}
