use crate::cli::game_updater::CliChannelEvent;
use crate::sudoku::algorithms::base_algorithms::BaseAlgorithms;
use crate::sudoku::algorithms::perf::PerfTracker;
use crate::sudoku::board::{CellType, SudokuBoard};
use std::sync::mpsc::Sender;

pub struct Backtracking<'a> {
    board: &'a mut SudokuBoard,
    editable_cells: Vec<(usize, usize)>,
    board_tx: Sender<CliChannelEvent>,
}

impl<'a> BaseAlgorithms<'a> for Backtracking<'a> {
    fn new(board: &'a mut SudokuBoard, board_tx: Sender<CliChannelEvent>) -> Self {
        let editable_cells = board.get_editable_cells();
        Backtracking {
            board,
            editable_cells,
            board_tx,
        }
    }

    fn resolve(self) -> PerfTracker {
        let this = self;
        let mut backtrack_index = 0usize;
        let mut perf = PerfTracker::new();

        perf.start();

        while this.editable_cells.len() > backtrack_index {
            let (mut x, mut y) = this.editable_cells[backtrack_index];
            let mut current_value = {
                let cell = this.board.find_cell_from_coordinates(x, y).unwrap();
                cell.value.or(Some(1))
            };
            let board = &mut *this.board;

            while current_value.unwrap() <= SudokuBoard::BOARD_MAX_NUMBER as CellType {
                if Self::update_and_incr(board, &mut perf, &this.board_tx, x, y, current_value) {
                    backtrack_index += 1;
                    break;
                } else {
                    if current_value.unwrap() >= SudokuBoard::BOARD_MAX_NUMBER as CellType {
                        let _ = board.set_cell(x, y, None).unwrap();
                        perf.incr();

                        backtrack_index -= 1;
                        (x, y) = this.editable_cells[backtrack_index];
                        let cell = board.find_cell_from_coordinates(x, y).unwrap();
                        current_value = if cell.value.is_some() {
                            Some(cell.value.unwrap() + 1)
                        } else {
                            Some(1)
                        };
                    } else {
                        current_value = Some(current_value.unwrap() + 1);
                    }
                }
            }

            if current_value.unwrap() > SudokuBoard::BOARD_MAX_NUMBER as CellType {
                let _ = board.set_cell(x, y, None).unwrap();
                perf.incr();
                backtrack_index -= 1;
            }
        }

        perf.finish();
        let result = this.board.validate_solution();

        if result.is_err() {
            panic!("{:?}", result)
        }

        perf
    }
}
