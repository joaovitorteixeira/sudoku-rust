use colored::Colorize;
use std::{fmt, sync::mpsc::Sender};

type Box = [[SudokuCell; 3]; 3];
type Board = [[Box; 3]; 3];

#[derive(Debug, Clone, Copy)]
pub struct SudokuCell {
    pub value: Option<u8>,
    editable: bool,
    pub x: usize,
    pub y: usize,
}

impl SudokuCell {
    fn new(value: Option<u8>) -> Self {
        SudokuCell {
            value,
            editable: if value.is_some() { false } else { true },
            x: 0,
            y: 0,
        }
    }

    pub fn as_mut_ptr(&self) -> Result<&mut SudokuCell, String> {
        let cell_ptr = self as *const SudokuCell as *mut SudokuCell;

        unsafe {
            cell_ptr
                .as_mut()
                .ok_or("Failed to get mutable reference to cell".to_string())
        }
    }
}
#[derive(Debug)]
pub struct SudokuBoard {
    board: Board,
    board_tx: Sender<String>,
}

impl SudokuBoard {
    const BOARD_DIVIDER: &str = " |";
    const BOARD_COLUMN_SIZE: u16 = 9;
    const BOARD_LENGTH: u16 =
        (Self::BOARD_COLUMN_SIZE * 2 - 1) + Self::BOARD_DIVIDER.len() as u16 * 2;

    fn initialize_box() -> Box {
        ([[SudokuCell::new(None); 3]; 3]).into()
    }

    fn initialize_board() -> Board {
        (
            [
                Self::initialize_box(),
                Self::initialize_box(),
                Self::initialize_box(),
            ],
            [
                Self::initialize_box(),
                Self::initialize_box(),
                Self::initialize_box(),
            ],
            [
                Self::initialize_box(),
                Self::initialize_box(),
                Self::initialize_box(),
            ],
        )
            .into()
    }

    pub fn new(list: Vec<Vec<Option<u8>>>, board_tx: Sender<String>) -> Result<Self, String> {
        if list.len() != 9 {
            return Err("The provided list must have 9 lines".to_string());
        }

        let sudoku_board: SudokuBoard = SudokuBoard {
            board: Self::initialize_board(),
            board_tx,
        };

        for (line_index, row) in list.iter().enumerate() {
            if row.len() != Self::BOARD_COLUMN_SIZE.into() {
                return Err("The provided list must have 9 lines".to_string());
            }

            for (column_index, value) in row.iter().enumerate() {
                let cell_result = sudoku_board.find_cell_from_coordinates(line_index, column_index);
                if let Ok(cell) = cell_result {
                    if let Ok(cell_ptr) = cell.as_mut_ptr() {
                        cell_ptr.value = *value;
                        cell_ptr.editable = value.is_none();
                        cell_ptr.x = line_index;
                        cell_ptr.y = column_index;
                    } else {
                        return Err("Unexpected error when casting".to_string());
                    }
                } else {
                    return Err(cell_result.unwrap_err());
                }
            }
        }

        Ok(sudoku_board)
    }

    fn find_box_from_coordinate(&self, x: usize, y: usize) -> &Box {
        let board_row_index = x / 3;
        let board_column_index = y / 3;
        let sudoku_box: &Box = &self.board[board_row_index][board_column_index];

        sudoku_box
    }

    pub fn find_cell_from_coordinates(&self, x: usize, y: usize) -> Result<&SudokuCell, String> {
        let box_row_index = x % 3;
        let box_column_index = y % 3;
        let sudoku_box: &Box = self.find_box_from_coordinate(x, y);
        let cell_result: Option<&SudokuCell> = Some(&sudoku_box[box_row_index][box_column_index]);

        if let Some(cell) = cell_result {
            Ok(cell)
        } else {
            Err(format!("Cell not found at coordinates ({}, {})", x, y))
        }
    }

    pub fn update_value(&mut self, x: usize, y: usize, value: Option<u8>) -> Result<(), String> {
        if x >= Self::BOARD_COLUMN_SIZE.into() || y >= Self::BOARD_COLUMN_SIZE.into() {
            return Err(format!("Invalid coordinates ({}, {})", x, y));
        }

        if !self.is_valid_insertion(x, y, value) {
            return Err("Invalid Insertion".to_string());
        }

        let cell_result = self.find_cell_from_coordinates(x, y);

        if let Ok(cell) = cell_result {
            if let Ok(cell_ptr) = cell.as_mut_ptr() {
                cell_ptr.value = value;

                let result = self.board_tx.send(format!("{:}", self));

                if let Some(_) = result.err() {
                    return Err("Channel is disconnected".into());
                }

                Ok(())
            } else {
                Err("You cannot edit this cell".to_string())
            }
        } else {
            Err(cell_result.unwrap_err())
        }
    }

    fn is_valid_insertion(&mut self, x: usize, y: usize, new_value: Option<u8>) -> bool {
        if let Some(value) = new_value {
            return self.is_valid_box(x, y, value)
                && self.is_valid_line(x, value)
                && self.is_valid_column(y, value);
        } else {
            return true;
        }
    }

    fn is_valid_box(&mut self, x: usize, y: usize, new_value: u8) -> bool {
        let sudoku_box = self.find_box_from_coordinate(x, y);
        sudoku_box.iter().all(|&lines| {
            let result = lines.iter().all(|&cell| cell.value != Some(new_value));
            return result;
        })
    }

    fn is_valid_line(&self, x: usize, new_value: u8) -> bool {
        for y in 0..Self::BOARD_COLUMN_SIZE.into() {
            let cell = self.find_cell_from_coordinates(x, y).unwrap();

            if cell.value == Some(new_value) {
                return false;
            }
        }

        return true;
    }

    fn is_valid_column(&self, y: usize, new_value: u8) -> bool {
        for x in 0..Self::BOARD_COLUMN_SIZE.into() {
            let cell = self.find_cell_from_coordinates(x, y).unwrap();

            if cell.value == Some(new_value) {
                return false;
            }
        }

        return true;
    }

    pub fn get_editable_cells(&self) -> Vec<SudokuCell> {
        let mut editable_cells = vec![];
        for x in 0..Self::BOARD_COLUMN_SIZE.into() {
            for y in 0..Self::BOARD_COLUMN_SIZE.into() {
                let cell = self.find_cell_from_coordinates(x, y).unwrap();

                if cell.editable {
                    editable_cells.push(*cell);
                }
            }
        }

        editable_cells
    }
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let mut line_index: usize = 0;
        let mut previous_board_row_index: Option<usize> = None;

        while line_index < Self::BOARD_COLUMN_SIZE.into() {
            let mut line_str = String::new();
            let board_row_index = line_index / 3;
            let mut column_index: usize = 0;

            while column_index < 3 {
                let sudoku_box = self.find_box_from_coordinate(line_index, column_index * 3);
                let box_line = &sudoku_box[line_index % 3];
                let box_line_str = box_line.iter().fold("".to_string(), |acc, cell| {
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

                    if acc.len() == 0 {
                        // Is first column?
                        format!("{acc}{value}")
                    } else {
                        format!("{acc} {value}")
                    }
                });

                line_str.push_str(&box_line_str);
                line_str.push_str(&" |".on_white());
                column_index += 1;
            }

            if previous_board_row_index != Some(board_row_index) {
                previous_board_row_index = Some(board_row_index);
                output.push_str(&"-".repeat(Self::BOARD_LENGTH.into()).on_white());
                output.push_str("\n");
            }

            output.push_str(&line_str);
            output.push_str("\n");

            line_index += 1;
        }

        write!(f, "{}", output)
    }
}
