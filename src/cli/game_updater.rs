use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::cli::sudoku_printer::SudokuPrinter;
use crate::sudoku::board::SudokuCell;

pub struct GameUpdater {
    board_rx: Receiver<CliChannelEvent>,
    throttle_ms: u64,
    sudoku: SudokuPrinter,
}

pub enum CliChannelEvent {
    Update(SudokuCell),
    ForceLastPrint,
}

impl GameUpdater {
    pub fn new(board_rx: Receiver<CliChannelEvent>, throttle_ms: Option<u64>) -> Self {
        GameUpdater {
            board_rx,
            throttle_ms: throttle_ms.or(Some(100)).unwrap(),
            sudoku: SudokuPrinter::new(),
        }
    }

    fn print(&self, message: String) -> String {
        print!("{}[2J", 27 as char);
        println!("{message}");

        message
    }

    fn format(&mut self, sudoku_message: SudokuCell) -> String {
        let cell = &mut self.sudoku.board[sudoku_message.x][sudoku_message.y];
        cell.value = sudoku_message.value;
        cell.editable = sudoku_message.editable;

        format!("{}", self.sudoku)
    }

    pub fn listen(&mut self) -> Result<(), String> {
        let mut last_print = Instant::now() - Duration::from_millis(self.throttle_ms);
        let interval = Duration::from_millis(self.throttle_ms);
        let mut last_message: Option<String> = None;

        loop {
            match self.board_rx.recv() {
                Ok(sudoku_message) => {
                    let now = Instant::now();
                    match sudoku_message {
                        CliChannelEvent::Update(sudoku_cell) => {
                            let out = self.format(sudoku_cell);

                            if now.duration_since(last_print) >= interval {
                                last_print = now;
                            }

                            last_message = Some(out);
                        }
                        CliChannelEvent::ForceLastPrint => break,
                    }
                }
                Err(_) => break,
            }
        }

        if let Some(message) = last_message {
            self.print(message);
        }

        Ok(())
    }
}
