use crate::sudoku::{
    algorithms::perf::PerfTracker,
    board::{CellType, SudokuBoard},
};

pub trait BaseAlgorithms<'a> {
    fn new(sudoku_board: &'a mut SudokuBoard) -> Self;
    fn resolve(self);

    fn update_and_incr(
        board: &mut SudokuBoard,
        perf: &mut PerfTracker,
        x: usize,
        y: usize,
        value: Option<CellType>,
    ) -> bool {
        let res = board.update_value(x, y, value);
        perf.incr();
        res.is_ok()
    }
}
