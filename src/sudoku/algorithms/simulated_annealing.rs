use std::{sync::mpsc::Sender, thread::sleep, time::Duration};

use crate::{
    cli::{board_notifier::broadcast_board, game_updater::CliChannelEvent},
    sudoku::{
        algorithms::{base_algorithms::BaseAlgorithms, perf::PerfTracker},
        board::SudokuBoard,
    },
};

pub struct SimulatedAnnealing<'a> {
    board: &'a mut SudokuBoard,
    board_tx: Sender<CliChannelEvent>,
}

impl<'a> BaseAlgorithms<'a> for SimulatedAnnealing<'a> {
    fn new(
        sudoku_board: &'a mut crate::sudoku::board::SudokuBoard,
        board_tx: std::sync::mpsc::Sender<CliChannelEvent>,
    ) -> Self {
        SimulatedAnnealing {
            board: sudoku_board,
            board_tx,
        }
    }

    fn resolve(mut self) -> super::perf::PerfTracker {
        let mut perf = PerfTracker::new();

        perf.start();
        self.random_initial_solution();

        while let Ok(cost) = self.board.calculate_raw_column_cost() {
            if cost == 0 {
                break;
            }
        }

        perf.finish();
        perf
    }
}

impl<'a> SimulatedAnnealing<'a> {
    fn random_initial_solution(&mut self) {
        for box_i in 0..SudokuBoard::BOARD_N {
            for box_j in 0..SudokuBoard::BOARD_N {
                let (mut fixed_values, empty_cells): (Vec<u16>, Vec<(usize, usize)>) = {
                    let cells = self.board.get_cells_from_box(box_i, box_j);
                    let fixed_values = cells.iter().filter_map(|cell| cell.value).collect();
                    let empty_cells = cells
                        .iter()
                        .filter(|cell| cell.value.is_none())
                        .map(|cell| (cell.x, cell.y))
                        .collect();
                    (fixed_values, empty_cells)
                };

                for (x, y) in empty_cells {
                    let value = self.get_random_value(&fixed_values);
                    self.board
                        .set_cell_unchecked(x, y, Some(value))
                        .expect("Failed to set cell");
                    fixed_values.push(value);
                    broadcast_board(self.board, &self.board_tx);
                }
            }
        }
    }

    fn get_random_value(&self, fixed_values: &[u16]) -> u16 {
        loop {
            let value = rand::random_range(1..=SudokuBoard::BOARD_MAX_NUMBER as u16);
            if !fixed_values.contains(&value) {
                return value;
            }
        }
    }
}
