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
    UpdateAll(Vec<SudokuCell>),
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

    fn update_cell(&mut self, update: SudokuCell) {
        let cell = &mut self.sudoku.board[update.x][update.y];
        cell.value = update.value;
        cell.editable = update.editable;
    }

    pub fn listen(&mut self) -> Result<(), String> {
        let mut last_print = Instant::now() - Duration::from_millis(self.throttle_ms);
        let interval = Duration::from_millis(self.throttle_ms);

        loop {
            match self.board_rx.recv() {
                Ok(sudoku_message) => {
                    let now = Instant::now();
                    match sudoku_message {
                        CliChannelEvent::Update(sudoku_cell) => {
                            self.update_cell(sudoku_cell);

                            if now.duration_since(last_print) >= interval {
                                self.print(format!("{}", self.sudoku));
                                last_print = now;
                            }
                        }
                        CliChannelEvent::UpdateAll(sudoku_cells) => {
                            for sudoku_cell in sudoku_cells {
                                self.update_cell(sudoku_cell);
                            }

                            self.print(format!("{}", self.sudoku));
                        }
                    }
                }
                Err(_) => break,
            }
        }

        self.print(format!("{}", self.sudoku));

        Ok(())
    }
}
