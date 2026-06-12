use std::sync::mpsc::Sender;

use crate::{cli::game_updater::CliChannelEvent, sudoku::board::SudokuBoard};

pub fn broadcast_board(board: &SudokuBoard, tx: &Sender<CliChannelEvent>) {
    let _ = tx.send(CliChannelEvent::UpdateAll(
        board.all_cells().into_iter().map(|cell| *cell).collect(),
    ));
}
