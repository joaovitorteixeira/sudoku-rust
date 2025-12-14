use crate::sudoku::board::{SudokuBoard, SudokuCell};
use colored::Colorize;
use std::fmt;

const BOARD_DIVIDER: &str = " |";
const BOARD_MAX_NUMBER: usize = SudokuBoard::BOARD_MAX_NUMBER;
const BOARD_N: usize = SudokuBoard::BOARD_N;
const BOARD_LENGTH: usize = (BOARD_MAX_NUMBER * 2 - 1) + BOARD_DIVIDER.len() * 2;

pub struct SudokuPrinter {
    pub board: [[SudokuCell; BOARD_MAX_NUMBER]; BOARD_MAX_NUMBER],
}

impl SudokuPrinter {
    pub fn new() -> Self {
        SudokuPrinter {
            board: [[SudokuCell {
                editable: false,
                value: None,
                x: 0,
                y: 0,
            }; 9]; 9],
        }
    }
}

impl fmt::Display for SudokuPrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let mut previous_board_row_index: Option<usize> = None;

        for x in 0..BOARD_MAX_NUMBER {
            let board_row_index = x / BOARD_N;
            let mut line_str = String::new();
            let mut box_line_str = String::new();

            for y in 0..BOARD_MAX_NUMBER {
                let cell = self.board[x][y];
                let value = match cell.value {
                    Some(match_value) => {
                        if cell.editable {
                            match_value.to_string().yellow()
                        } else {
                            match_value.to_string().blue()
                        }
                    }
                    None => "?".to_string().red(),
                };
                box_line_str.push_str(&format!("{value} "));

                if (y + 1) % 3 == 0 {
                    line_str.push_str(&"|".white());
                    line_str.push_str(&box_line_str);
                    box_line_str.clear();
                }
            }

            if previous_board_row_index != Some(board_row_index) {
                previous_board_row_index = Some(board_row_index);
                output.push_str(&"-".repeat(BOARD_LENGTH.into()).on_white());
                output.push_str("\n");
            }

            output.push_str(&line_str);
            output.push_str("\n");
        }

        write!(f, "{}", output)
    }
}
