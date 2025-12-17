use std::sync::mpsc::Sender;

use crate::cli::game_updater::CliChannelEvent;

const BOARD_N: usize = 3;

type Box = [[SudokuCell; BOARD_N]; BOARD_N];
type Board = [[Box; BOARD_N]; BOARD_N];

#[derive(Debug, Clone, Copy)]
pub struct SudokuCell {
    pub value: Option<u8>,
    pub editable: bool,
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
    board_tx: Sender<CliChannelEvent>,
}

impl SudokuBoard {
    pub const BOARD_N: usize = BOARD_N;
    pub const BOARD_MAX_NUMBER: usize = Self::BOARD_N.pow(2);

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

    pub fn new(
        list: Vec<Vec<Option<u8>>>,
        board_tx: Sender<CliChannelEvent>,
    ) -> Result<Self, String> {
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

    fn decompose_coordinates(x: usize, y: usize) -> (usize, usize, usize, usize) {
        let board_row_index = x / Self::BOARD_N;
        let board_column_index = y / Self::BOARD_N;
        let box_row_index = x % Self::BOARD_N;
        let box_column_index = y % Self::BOARD_N;
        (
            board_row_index,
            board_column_index,
            box_row_index,
            box_column_index,
        )
    }

    pub fn find_cell_from_coordinates(&self, x: usize, y: usize) -> Result<&SudokuCell, String> {
        let decomposed_coordinates = Self::decompose_coordinates(x, y);
        let sudoku_box: &Box = &self.board[decomposed_coordinates.0][decomposed_coordinates.1];
        let cell_result: Option<&SudokuCell> =
            Some(&sudoku_box[decomposed_coordinates.2][decomposed_coordinates.3]);

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
        let decomposed_coordinates = Self::decompose_coordinates(x, y);
        let sudoku_box: &mut Box =
            &mut self.board[decomposed_coordinates.0][decomposed_coordinates.1];
        let cell_result: Option<&mut SudokuCell> =
            Some(&mut sudoku_box[decomposed_coordinates.2][decomposed_coordinates.3]);

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

        let result = {
            match self.find_cell_from_coordinates_mut(x, y) {
                Ok(cell_ptr) => {
                    cell_ptr.value = value;

                    Ok(())
                }
                Err(e) => Err(e),
            }
        };

        result
    }

    pub fn finish(&self) {
        for x in 0..Self::BOARD_MAX_NUMBER {
            for y in 0..Self::BOARD_MAX_NUMBER {
                let cell_result = self.find_cell_from_coordinates(x, y);

                if let Ok(cell) = cell_result {
                    let _ = self.board_tx.send(CliChannelEvent::Update(*cell));
                }
            }
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
        let decomposed_coordinates = Self::decompose_coordinates(x, y);
        let sudoku_box = self.board[decomposed_coordinates.0][decomposed_coordinates.1];
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
