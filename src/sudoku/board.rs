use colored::Colorize;
use std::fmt;

pub type Quadrant = [[Option<u8>; 3]; 3];
pub type BoardLine = [Quadrant; 3];
pub type Board = [BoardLine; 3];

#[derive(Debug)]
pub struct SudokuBoard {
    board: Board,
}

impl SudokuBoard {
    const BOARD_DIVIDER: &str = " |";
    const BOARD_COLUMN_SIZE: u16 = 9;
    const BOARD_LENGTH: u16 =
        (Self::BOARD_COLUMN_SIZE * 2 - 1) + Self::BOARD_DIVIDER.len() as u16 * 2;

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

    pub fn new(list: Vec<Vec<Option<u8>>>) -> Result<Self, String> {
        if list.len() != 9 {
            return Err("The provided list must have 9 lines".to_string());
        }

        let mut sudoku_board: SudokuBoard = SudokuBoard {
            board: Self::initialize_board(),
        };

        for (line_index, row) in list.iter().enumerate() {
            if row.len() != 9 {
                return Err("The provided list must have 9 lines".to_string());
            }

            let board_line_index = (((line_index / 3) as f32).floor()) as usize;
            let board_line: &mut BoardLine = &mut sudoku_board.board[board_line_index];

            for (column_index, value) in row.iter().enumerate() {
                let quadrant_index =
                    (f32::trunc((column_index as f32 / 3.0) * 10.0) / 10.0).floor() as usize;
                let quadrant_column_index = column_index - quadrant_index * 3;
                let quadrant_line_index = line_index - board_line_index * 3;
                let quadrant: &mut Quadrant = &mut board_line[quadrant_line_index];

                quadrant[quadrant_index][quadrant_column_index] = *value;
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

            while column_index < 3 {
                let quadrant_index = line_index - board_line_index * 3;
                let quadrant_line = board_line[quadrant_index][column_index];
                let quadrant_line_str = quadrant_line.iter().fold("".to_string(), |acc, value| {
                    let value = match value {
                        Some(match_value) => match_value.to_string().blue(),
                        None => "?".to_string().red(),
                    };

                    if acc.len() == 0 {
                        // Is first column?
                        format!("{acc}{value}")
                    } else {
                        format!("{acc} {value}")
                    }
                });

                line_str.push_str(&quadrant_line_str);
                line_str.push_str(&" |".on_white());
                column_index += 1;
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
