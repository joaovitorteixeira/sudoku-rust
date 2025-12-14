use crate::sudoku::board::SudokuBoard;

pub trait BaseAlgorithms<'a> {
    fn new(sudoku_board: &'a mut SudokuBoard) -> Self;
    fn resolve(self);
}
