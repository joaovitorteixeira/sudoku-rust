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
    editable_by_box: [[Vec<(usize, usize)>; SudokuBoard::BOARD_N]; SudokuBoard::BOARD_N],
    board_tx: Sender<CliChannelEvent>,
    perf: PerfTracker,
    temperature: f64,
    markov_chain_length: usize,
}

impl<'a> BaseAlgorithms<'a> for SimulatedAnnealing<'a> {
    fn new(
        sudoku_board: &'a mut crate::sudoku::board::SudokuBoard,
        board_tx: std::sync::mpsc::Sender<CliChannelEvent>,
    ) -> Self {
        let editable_by_box = std::array::from_fn(|box_i| {
            std::array::from_fn(|box_j| {
                sudoku_board
                    .get_cells_from_box(box_i, box_j)
                    .into_iter()
                    .filter(|c| c.editable)
                    .map(|c| (c.x, c.y))
                    .collect()
            })
        });
        let markov_chain_length = Self::markov_chain_length(&editable_by_box);

        SimulatedAnnealing {
            board: sudoku_board,
            board_tx,
            editable_by_box,
            perf: PerfTracker::new(),
            temperature: 1.0,
            markov_chain_length,
        }
    }

    fn resolve(mut self) -> super::perf::PerfTracker {
        self.perf.start();
        self.random_initial_solution();

        let mut cost = self
            .board
            .calculate_raw_column_cost()
            .expect("Failed to calculate cost");

        self.temperature = self.estimate_initial_temperature(cost);
        cost = self
            .board
            .calculate_raw_column_cost()
            .expect("Failed to calculate cost");

        loop {
            let ((x1, y1), (x2, y2)) = self.get_random_swap_pair();
            let (old_value1, old_value2) = self
                .swap_cells(x1, y1, x2, y2)
                .expect("Failed to swap cells");

            if let Ok(new_cost) = self.board.calculate_raw_column_cost() {
                let delta = new_cost - cost;

                let accept = if delta <= 0 {
                    true
                } else {
                    rand::random::<f64>() < (-(delta as f64) / self.temperature).exp()
                };

                if accept {
                    cost = new_cost;
                    if cost <= 0 {
                        break;
                    }
                } else {
                    self.board
                        .set_cell_unchecked(x1, y1, old_value1)
                        .expect("Failed to set cell");
                    self.board
                        .set_cell_unchecked(x2, y2, old_value2)
                        .expect("Failed to set cell");
                }
            } else {
                panic!("Failed to calculate cost");
            }
        }

        self.perf.finish();

        self.board
            .validate_solution()
            .expect("Failed to validate solution");

        self.perf
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

    fn get_random_swap_pair(&self) -> ((usize, usize), (usize, usize)) {
        loop {
            let (box_i, box_j) = (
                rand::random_range(0..SudokuBoard::BOARD_N),
                rand::random_range(0..SudokuBoard::BOARD_N),
            );
            let editable_cells = &self.editable_by_box[box_i][box_j];
            if editable_cells.len() < 2 {
                continue;
            }

            let (x1, y1) = editable_cells[rand::random_range(0..editable_cells.len())];
            let (x2, y2) = editable_cells[rand::random_range(0..editable_cells.len())];

            if (x1, y1) == (x2, y2) {
                continue;
            }

            return ((x1, y1), (x2, y2));
        }
    }

    fn swap_cells(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
    ) -> Result<((Option<u16>, Option<u16>)), String> {
        let (old_value1, old_value2) = {
            let cell1 = self
                .board
                .find_cell_from_coordinates(x1, y1)
                .expect("Failed to find cell");
            let cell2 = self
                .board
                .find_cell_from_coordinates(x2, y2)
                .expect("Failed to find cell");
            (cell1.value, cell2.value)
        };
        self.board
            .set_cell_unchecked(x1, y1, old_value2)
            .expect("Failed to set cell");
        self.board
            .set_cell_unchecked(x2, y2, old_value1)
            .expect("Failed to set cell");

        self.perf.incr();

        Ok((old_value1, old_value2))
    }

    fn estimate_initial_temperature(&mut self, mut cost: u16) -> f64 {
        const SAMPLE_SIZE: usize = 100;
        let mut deltas = Vec::with_capacity(SAMPLE_SIZE);

        for _ in 0..SAMPLE_SIZE {
            let ((x1, y1), (x2, y2)) = self.get_random_swap_pair();
            let (old_value1, old_value2) = self
                .swap_cells(x1, y1, x2, y2)
                .expect("Failed to swap cells");
            let new_cost = self
                .board
                .calculate_raw_column_cost()
                .expect("Failed to calculate cost");
            let delta = new_cost as i32 - cost as i32;
            deltas.push(delta as f64);

            self.board
                .set_cell_unchecked(x1, y1, old_value1)
                .expect("Failed to set cell");
            self.board
                .set_cell_unchecked(x2, y2, old_value2)
                .expect("Failed to set cell");
        }

        self.std_dev(&deltas).max(1.0)
    }

    fn std_dev(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 1.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    fn markov_chain_length(
        editable_by_box: &[[Vec<(usize, usize)>; SudokuBoard::BOARD_N]; SudokuBoard::BOARD_N],
    ) -> usize {
        editable_by_box
            .iter()
            .flat_map(|row| row.iter())
            .map(|cells| cells.len() * cells.len())
            .sum()
    }
}
