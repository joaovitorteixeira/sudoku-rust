use crate::sudoku::{
    algorithms::{base_algorithms::BaseAlgorithms, perf::PerfTracker},
    board::SudokuBoard,
};

struct EditableCells {
    x: usize,
    y: usize,
    candidates: Vec<u8>,
}

pub struct CandidateElection<'a> {
    board: &'a mut SudokuBoard,
    editable_cells: Vec<EditableCells>,
}

impl<'a> CandidateElection<'a> {
    fn update_and_incr(
        &mut self,
        perf: &mut PerfTracker,
        x: usize,
        y: usize,
        value: Option<u8>,
    ) -> bool {
        let res = self.board.update_value(x, y, value);
        perf.incr();
        res.is_ok()
    }
}

impl<'a> BaseAlgorithms<'a> for CandidateElection<'a> {
    fn new(sudoku_board: &'a mut SudokuBoard) -> Self {
        let cells = sudoku_board.get_editable_cells();
        let mut editable_cells = Vec::with_capacity(cells.len());

        for (x, y) in cells {
            let candidates: Vec<u8> = (1..=SudokuBoard::BOARD_MAX_NUMBER as u8)
                .filter(|&v| sudoku_board.is_valid_insertion(x, y, Some(v as u8)))
                .collect();

            editable_cells.push(EditableCells { candidates, x, y });
        }

        CandidateElection {
            board: sudoku_board,
            editable_cells,
        }
    }

    fn resolve(self) {
        let mut this = self;
        let mut backtrack_index = 0usize;
        let mut perf = PerfTracker::new();

        perf.start();

        while this.editable_cells.len() > backtrack_index {
            let (mut x, mut y, mut candidates) = {
                let ec = &this.editable_cells[backtrack_index];
                (ec.x, ec.y, ec.candidates.clone())
            };
            let cell = this.board.find_cell_from_coordinates(x, y).unwrap();
            let mut index = if cell.value.is_some() {
                candidates
                    .iter()
                    .position(|value| *value == cell.value.unwrap())
                    .unwrap()
                    + 1
            } else {
                0
            };

            while index < candidates.len() {
                let value = candidates[index];

                if this.update_and_incr(&mut perf, x, y, Some(value)) {
                    backtrack_index += 1;
                    break;
                } else {
                    if index >= candidates.len() {
                        let _ = this.update_and_incr(&mut perf, x, y, None);

                        backtrack_index -= 1;
                        (x, y, candidates) = {
                            let ec = &this.editable_cells[backtrack_index];
                            (ec.x, ec.y, ec.candidates.clone())
                        };
                        let cell = this.board.find_cell_from_coordinates(x, y).unwrap();

                        if cell.value.is_some() {
                            index = candidates
                                .iter()
                                .position(|value| *value == cell.value.unwrap())
                                .unwrap()
                                + 1;
                        } else {
                            index = 0
                        };
                    } else {
                        index += 1;
                    }
                }
            }

            if index >= candidates.len() {
                let _ = this.board.update_value(x, y, None).unwrap();
                perf.incr();
                backtrack_index -= 1;
            }
        }

        perf.finish();
        this.board.finish();
        perf.print_summary();
    }
}
