use std::sync::mpsc::Sender;

use crate::{cli::game_updater::CliChannelEvent, sudoku::board::SudokuBoard};

pub fn broadcast_board(board: &SudokuBoard, tx: &Sender<CliChannelEvent>) {
    for cell in board.all_cells() {
        let _ = tx.send(CliChannelEvent::Update(*cell));
    }
}
