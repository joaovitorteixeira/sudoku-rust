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

    fn unset_cell(
        board: &mut SudokuBoard,
        board_tx: &Sender<CliChannelEvent>,
        perf: &mut PerfTracker,
        x: usize,
        y: usize,
        sleep_ms: Option<u64>,
    ) -> bool {
        let res = board.set_cell(x, y, None);
        let mut is_ok: bool = true;
        match res {
            Ok(cell) => {
                let _ = board_tx.send(CliChannelEvent::Update(cell));
            }
            Err(_) => is_ok = false,
        };

        perf.incr();
        perf.sleep(sleep_ms);
        is_ok
    }

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
        let mut is_ok = true;
        perf.incr();

        match res {
            Ok(cell) => {
                let _ = board_tx.send(CliChannelEvent::Update(cell));
            }
            Err(_) => {
                is_ok = false;
            }
        };

        perf.sleep(sleep_ms);
        is_ok
    }
}
