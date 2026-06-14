use std::sync::mpsc::Sender;

use crate::{
    cli::game_updater::CliChannelEvent,
    sudoku::{
        algorithms::perf::PerfTracker,
        board::{CellType, SudokuBoard},
    },
};

pub trait BaseAlgorithms<'a> {
    fn new(
        sudoku_board: &'a mut SudokuBoard,
        board_tx: Sender<CliChannelEvent>,
        sleep_ms: Option<u64>,
    ) -> Self;
    fn resolve(self) -> PerfTracker;

    fn update_and_incr(
        board: &mut SudokuBoard,
        perf: &mut PerfTracker,
        board_tx: &Sender<CliChannelEvent>,
        x: usize,
        y: usize,
        value: Option<CellType>,
        sleep_ms: Option<u64>,
    ) -> bool {
        let res = board.set_cell(x, y, value);
        perf.incr();

        let is_ok = res.is_ok();

        if is_ok {
            if let Ok(cell) = board.find_cell_from_coordinates(x, y) {
                let _ = board_tx.send(CliChannelEvent::Update(*cell));
            }
        }

        perf.sleep(sleep_ms);
        is_ok
    }
}
