use std::{thread, time::Duration};

use crate::sudoku::board::{SudokuBoard};

pub struct Backtracking<'a> {
    board: &'a mut SudokuBoard,
    editable_cells: Vec<(usize, usize)>,
}

impl<'a> Backtracking<'a> {
    pub fn new(board: &'a mut SudokuBoard) -> Self {
        let editable_cells = board.get_editable_cells();
        Backtracking {
            board,
            editable_cells,
        }
    }

    pub fn resolve(self) {
        let mut backtrack_index = 0usize;

        while self.editable_cells.len() > backtrack_index {
            let (mut x, mut y) = self.editable_cells[backtrack_index];
            let mut current_value = {
                let cell = self.board.find_cell_from_coordinates(x, y).unwrap();
                cell.value.or(Some(1))
            };

            while current_value.unwrap() <= 9 {
                thread::sleep(Duration::from_millis(1));
                match self.board.update_value(x, y, current_value) {
                    Ok(_) => {
                        backtrack_index += 1;
                        break;
                    }
                    Err(_) => {
                        if current_value.unwrap() >= 9 {
                            let _ = self.board.update_value(x, y, None).unwrap();

                            backtrack_index -= 1;
                            (x, y) = self.editable_cells[backtrack_index];
                            let cell = self.board.find_cell_from_coordinates(x, y).unwrap();
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
            }

            if current_value.unwrap() > 9 {
                let _ = self.board.update_value(x, y, None).unwrap();
                backtrack_index -= 1;
            }
        }
    }
}
