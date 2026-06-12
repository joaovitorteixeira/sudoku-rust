use std::sync::mpsc::Sender;

use crate::{
    cli::game_updater::CliChannelEvent,
    sudoku::{
        algorithms::perf::PerfTracker,
        board::{CellType, SudokuBoard, SudokuCell},
    },
};

pub trait BaseAlgorithms<'a> {
    fn new(sudoku_board: &'a mut SudokuBoard, board_tx: Sender<CliChannelEvent>) -> Self;
    fn resolve(self);

    fn update_and_incr(
        board: &mut SudokuBoard,
        perf: &mut PerfTracker,
        board_tx: &Sender<CliChannelEvent>,
        x: usize,
        y: usize,
        value: Option<CellType>,
    ) -> bool {
        let res = board.update_value(x, y, value);
        perf.incr();

        let is_ok = res.is_ok();

        if is_ok {
            let _ = board_tx.send(CliChannelEvent::Update(SudokuCell {
                value: value,
                editable: true,
                x,
                y,
            }));
        }

        is_ok
    }
}
