use std::sync::mpsc::Sender;

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

        let initial_temperature = self.estimate_initial_temperature(cost);
        self.temperature = initial_temperature;

        const ALPHA: f64 = 0.99;
        const MIN_TEMPERATURE: f64 = 0.01;
        const REHEAT_AFTER: usize = 20;
        const MAX_REHEATS: usize = 100;

        let mut chains_without_improvement = 0;
        let mut best_cost = cost;
        let mut reheat_count = 0;

        'search: loop {
            for _ in 0..self.markov_chain_length {
                cost = self.try_one_move(cost).expect("Failed to try one move");

                if cost == 0 {
                    break 'search;
                }
            }

            if cost < best_cost {
                best_cost = cost;
                chains_without_improvement = 0;
            } else {
                chains_without_improvement += 1;
            }

            self.temperature *= ALPHA;

            let should_reheat =
                self.temperature < MIN_TEMPERATURE || chains_without_improvement >= REHEAT_AFTER;

            if should_reheat {
                if reheat_count >= MAX_REHEATS {
                    break 'search;
                }

                reheat_count += 1;
                self.temperature = initial_temperature;
                self.random_initial_solution();
                cost = self
                    .board
                    .calculate_raw_column_cost()
                    .expect("Failed to calculate cost");
                best_cost = cost;
                chains_without_improvement = 0;
            }
        }

        self.perf.finish();

        match self.board.validate_solution() {
            Ok(()) => broadcast_board(self.board, &self.board_tx),
            Err(message) => {
                panic!("SA did not find a solution (cost={cost}): {message}");
            }
        }

        self.perf
    }
}

impl<'a> SimulatedAnnealing<'a> {
    fn random_initial_solution(&mut self) {
        for box_i in 0..SudokuBoard::BOARD_N {
            for box_j in 0..SudokuBoard::BOARD_N {
                let cells = self.board.get_cells_from_box(box_i, box_j);
                let mut fixed_values: Vec<u16> = cells
                    .iter()
                    .filter(|cell| !cell.editable)
                    .filter_map(|cell| cell.value)
                    .collect();

                for &(x, y) in &self.editable_by_box[box_i][box_j] {
                    let value = self.get_random_value(&fixed_values);
                    self.board
                        .set_cell_unchecked(x, y, Some(value))
                        .expect("Failed to set cell");
                    fixed_values.push(value);
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
        record_perf: bool,
    ) -> (Option<u16>, Option<u16>) {
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

        if record_perf {
            self.perf.incr();
        }

        (old_value1, old_value2)
    }

    /// Row/column cost for lines touched by a same-box swap.
    fn affected_lines_cost(
        &self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
    ) -> Result<u16, String> {
        let mut cost = self.board.row_cost(x1)?;
        cost = cost
            .checked_add(self.board.column_cost(y1)?)
            .ok_or_else(|| "Cost overflow".to_string())?;

        if x2 != x1 {
            cost = cost
                .checked_add(self.board.row_cost(x2)?)
                .ok_or_else(|| "Cost overflow".to_string())?;
        }
        if y2 != y1 {
            cost = cost
                .checked_add(self.board.column_cost(y2)?)
                .ok_or_else(|| "Cost overflow".to_string())?;
        }

        Ok(cost)
    }

    /// Update total row/column cost after a swap (board must already be swapped).
    fn cost_after_swap(
        &self,
        cost: u16,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        old_affected_cost: u16,
    ) -> Result<u16, String> {
        let new_affected_cost = self.affected_lines_cost(x1, y1, x2, y2)?;
        cost.checked_sub(old_affected_cost)
            .and_then(|c| c.checked_add(new_affected_cost))
            .ok_or_else(|| "Cost overflow".to_string())
    }

    fn estimate_initial_temperature(&mut self, cost: u16) -> f64 {
        const SAMPLE_SIZE: usize = 100;
        let mut deltas = Vec::with_capacity(SAMPLE_SIZE);

        for _ in 0..SAMPLE_SIZE {
            let ((x1, y1), (x2, y2)) = self.get_random_swap_pair();
            let old_affected = self
                .affected_lines_cost(x1, y1, x2, y2)
                .expect("Failed to calculate affected cost");
            let (old_value1, old_value2) = self.swap_cells(x1, y1, x2, y2, false);
            let new_cost = self
                .cost_after_swap(cost, x1, y1, x2, y2, old_affected)
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

    fn try_one_move(&mut self, cost: u16) -> Result<u16, String> {
        let ((x1, y1), (x2, y2)) = self.get_random_swap_pair();
        let old_affected = self.affected_lines_cost(x1, y1, x2, y2)?;
        let (old_value1, old_value2) = self.swap_cells(x1, y1, x2, y2, true);
        let new_cost = self.cost_after_swap(cost, x1, y1, x2, y2, old_affected)?;
        let delta = new_cost as i32 - cost as i32;

        let accept = if delta <= 0 {
            true
        } else {
            rand::random::<f64>() < (-(delta as f64) / self.temperature).exp()
        };

        if accept {
            Ok(new_cost)
        } else {
            self.board.set_cell_unchecked(x1, y1, old_value1)?;
            self.board.set_cell_unchecked(x2, y2, old_value2)?;
            Ok(cost)
        }
    }
}
