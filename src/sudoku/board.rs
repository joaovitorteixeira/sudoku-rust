use colored::Colorize;
use std::{fmt, sync::mpsc::Sender};

const BOARD_N: usize = 3;

type Box = [[SudokuCell; BOARD_N]; BOARD_N];
type Board = [[Box; BOARD_N]; BOARD_N];

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
}
#[derive(Debug)]
pub struct SudokuBoard {
    board: Board,
    board_tx: Sender<String>,
}

impl SudokuBoard {
    const BOARD_DIVIDER: &str = " |";
    pub const BOARD_N: usize = BOARD_N;
    pub const BOARD_MAX_NUMBER: usize = Self::BOARD_N.pow(2);
    const BOARD_LENGTH: usize = (Self::BOARD_MAX_NUMBER * 2 - 1) + Self::BOARD_DIVIDER.len() * 2;

    fn initialize_box() -> Box {
        ([[SudokuCell::new(None); Self::BOARD_N]; Self::BOARD_N]).into()
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
        if list.len() != Self::BOARD_MAX_NUMBER {
            return Err("The provided list must have 9 lines".to_string());
        }

        let mut sudoku_board: SudokuBoard = SudokuBoard {
            board: Self::initialize_board(),
            board_tx,
        };

        for (line_index, row) in list.iter().enumerate() {
            if row.len() != Self::BOARD_MAX_NUMBER.into() {
                return Err("The provided list must have 9 lines".to_string());
            }

            for (column_index, value) in row.iter().enumerate() {
                let cell_result =
                    sudoku_board.find_cell_from_coordinates_mut(line_index, column_index);
                if let Ok(cell) = cell_result {
                    cell.value = *value;
                    cell.editable = value.is_none();
                    cell.x = line_index;
                    cell.y = column_index;
                } else {
                    return Err(cell_result.unwrap_err());
                }
            }
        }

        Ok(sudoku_board)
    }

    fn find_box_from_coordinate(&self, x: usize, y: usize) -> &Box {
        let board_row_index = x / Self::BOARD_N;
        let board_column_index = y / Self::BOARD_N;
        let sudoku_box: &Box = &self.board[board_row_index][board_column_index];

        sudoku_box
    }

    fn find_box_from_coordinate_mut(&mut self, x: usize, y: usize) -> &mut Box {
        let board_row_index = x / Self::BOARD_N;
        let board_column_index = y / Self::BOARD_N;
        &mut self.board[board_row_index][board_column_index]
    }

    pub fn find_cell_from_coordinates(&self, x: usize, y: usize) -> Result<&SudokuCell, String> {
        let box_row_index = x % Self::BOARD_N;
        let box_column_index = y % Self::BOARD_N;
        let sudoku_box: &Box = self.find_box_from_coordinate(x, y);
        let cell_result: Option<&SudokuCell> = Some(&sudoku_box[box_row_index][box_column_index]);

        if let Some(cell) = cell_result {
            Ok(cell)
        } else {
            Err(format!("Cell not found at coordinates ({}, {})", x, y))
        }
    }

    fn find_cell_from_coordinates_mut(
        &mut self,
        x: usize,
        y: usize,
    ) -> Result<&mut SudokuCell, String> {
        let box_row_index = x % Self::BOARD_N;
        let box_column_index = y % Self::BOARD_N;
        let sudoku_box: &mut Box = self.find_box_from_coordinate_mut(x, y);
        let cell_result: Option<&mut SudokuCell> =
            Some(&mut sudoku_box[box_row_index][box_column_index]);

        if let Some(cell) = cell_result {
            Ok(cell)
        } else {
            Err(format!("Cell not found at coordinates ({}, {})", x, y))
        }
    }

    pub fn update_value(&mut self, x: usize, y: usize, value: Option<u8>) -> Result<(), String> {
        if x >= Self::BOARD_MAX_NUMBER.into() || y >= Self::BOARD_MAX_NUMBER.into() {
            return Err(format!("Invalid coordinates ({}, {})", x, y));
        }

        if !self.is_valid_insertion(x, y, value) {
            return Err("Invalid Insertion".to_string());
        }

        match self.find_cell_from_coordinates_mut(x, y) {
            Ok(cell_ptr) => {
                cell_ptr.value = value;

                let result = self.board_tx.send(format!("{:}", self));

                if let Some(_) = result.err() {
                    return Err("Channel is disconnected".into());
                }

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_valid_insertion(&self, x: usize, y: usize, new_value: Option<u8>) -> bool {
        if let Some(value) = new_value {
            return self.is_valid_box(x, y, value)
                && self.is_valid_line(x, value)
                && self.is_valid_column(y, value);
        } else {
            return true;
        }
    }

    fn is_valid_box(&self, x: usize, y: usize, new_value: u8) -> bool {
        let sudoku_box = self.find_box_from_coordinate(x, y);
        sudoku_box.iter().all(|&lines| {
            let result = lines.iter().all(|&cell| cell.value != Some(new_value));
            return result;
        })
    }

    fn is_valid_line(&self, x: usize, new_value: u8) -> bool {
        for y in 0..Self::BOARD_MAX_NUMBER.into() {
            let cell = self.find_cell_from_coordinates(x, y).unwrap();

            if cell.value == Some(new_value) {
                return false;
            }
        }

        return true;
    }

    fn is_valid_column(&self, y: usize, new_value: u8) -> bool {
        for x in 0..Self::BOARD_MAX_NUMBER.into() {
            let cell = self.find_cell_from_coordinates(x, y).unwrap();

            if cell.value == Some(new_value) {
                return false;
            }
        }

        return true;
    }

    pub fn get_editable_cells(&self) -> Vec<(usize, usize)> {
        let mut editable_cells = vec![];
        for x in 0..Self::BOARD_MAX_NUMBER.into() {
            for y in 0..Self::BOARD_MAX_NUMBER.into() {
                let cell = self.find_cell_from_coordinates(x, y).unwrap();

                if cell.editable {
                    editable_cells.push((cell.x, cell.y));
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

        while line_index < Self::BOARD_MAX_NUMBER.into() {
            let mut line_str = String::new();
            let board_row_index = line_index / Self::BOARD_N;
            let mut column_index: usize = 0;

            while column_index < 3 {
                let sudoku_box = self.find_box_from_coordinate(line_index, column_index * 3);
                let box_line = &sudoku_box[line_index % Self::BOARD_N];
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
