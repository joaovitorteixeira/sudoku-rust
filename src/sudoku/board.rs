use std::fmt;
use colored::Colorize;

pub type Quadrant = [[Option<u8>; 3]; 3];
pub type BoardLine = [Quadrant; 3];
pub type Board = [BoardLine; 3];

#[derive(Debug)]
pub struct SudokuBoard {
    board: Board,
}

impl SudokuBoard {
    const BOARD_DIVIDER: &str = " |";
    const BOARD_LENGTH: u16 = 9 * 2 + Self::BOARD_DIVIDER.len() as u16 * 3;

    fn initialize_quadrant() -> Quadrant {
        ([None, None, None], [None, None, None], [None, None, None]).into()
    }

    fn initialize_board() -> Board {
        (
            [
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
            ],
            [
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
            ],
            [
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
                Self::initialize_quadrant(),
            ],
        )
            .into()
    }

    fn find_quadrant_index_from_column_index(column: usize) -> usize {
        (f32::trunc((column as f32 / 3.0) * 10.0) / 10.0).floor() as usize
    }

    pub fn new(list: Vec<&[Option<u8>]>) -> Result<Self, String> {
        if list.len() != 9 {
            return Err("The provided list must have 9 lines".to_string());
        }

        let sudoku_board: SudokuBoard = SudokuBoard {
            board: Self::initialize_board(),
        };

        for (line_index, row) in list.iter().enumerate() {
            let board_line_index = (((line_index / 3) as f32).floor()) as usize;
            let board_line: BoardLine = sudoku_board.board[board_line_index];

            if row.len() != 9 {
                return Err("The provided list must have 9 lines".to_string());
            }

            for (column_index, cell) in row.iter().enumerate() {
                let quadrant_column_index = ((column_index / 3) as f32).floor() as usize;
                let quadrant_index = Self::find_quadrant_index_from_column_index(column_index);
                let mut quadrant = board_line[quadrant_index];

                quadrant[quadrant_index][quadrant_column_index] = *cell;
            }
        }

        Ok(sudoku_board)
    }
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let mut line_index: usize = 0;
        let mut previous_board_line_index: Option<usize> = None;

        while line_index < 9 {
            let board_line_index = (line_index as f32 / 3.0).floor() as usize;
            let board_line: BoardLine = self.board[board_line_index];
            let mut column_index: usize = 0;
            let mut line_str = String::new();

            while column_index < 9 {
                let quadrant_column_index = (column_index as f32 / 3.0).floor() as usize;
                let quadrant_index: usize =
                    Self::find_quadrant_index_from_column_index(column_index);
                let quadrant = board_line[quadrant_index][quadrant_column_index];
                let quadrant_line_str = quadrant.iter().fold("".to_string(), |acc, value| {
                    let value = match value {
                        Some(match_value) => match_value.to_string().blue(),
                        None => "?".to_string().red(),
                    };
                    format!("{acc} {value}")
                });

                line_str.push_str(&quadrant_line_str);
                line_str.push_str(&" |".on_white());
                column_index += 3;
            }

            if previous_board_line_index != Some(board_line_index) {
                previous_board_line_index = Some(board_line_index);
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
