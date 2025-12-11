use std::fmt;

pub type Quadrant = [[Option<u8>; 3]; 3];
pub type Board = [[Quadrant; 3]; 3];

#[derive(Debug)]
pub struct SudokuBoard {
    board: Board,
}

impl SudokuBoard {
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

    pub fn new(list: Vec<&[Option<u8>]>) -> Result<Self, String> {
        if list.len() != 9 {
            return Err("The provided list must have 9 lines".to_string());
        }

        let sudoku_board: SudokuBoard = SudokuBoard {
            board: Self::initialize_board(),
        };

        for (row_index, row) in list.iter().enumerate() {
            let sudoku_line_index = (((row_index / 3) as f32).floor()) as usize;
            let quadrant_line = sudoku_board.board[sudoku_line_index];

            if row.len() != 9 {
                return Err("The provided list must have 9 lines".to_string());
            }

            for (cell_index, cell) in row.iter().enumerate() {
                let quadrant_column_index = ((cell_index / 3) as f32).floor() as usize;
                let quadrant_index = ((f32::trunc((cell_index as f32 / 3.0) * 10.0) / 10.0)).floor() as usize; // TODO: create function findQuadrantIndex
                let mut quadrant = quadrant_line[quadrant_index];

                quadrant[quadrant_index][quadrant_column_index] = *cell;
            }
        }

        Ok(sudoku_board)
    }
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let mut line: u8 = 0;

        while line < 9 {
            let quadrant_line_index = (line as f32 / 3.0).floor() as usize;
            let board_line = self.board[quadrant_line_index];
            let mut column: u8 = 0;
            let mut line_str = String::new();

            while column < 9 {
                let quadrant_column_index = (column as f32 / 3.0).floor() as usize;
                let quadrant_index: usize = f32::trunc((column as f32 / 3.0) * 10.0 / 10.0).floor() as usize; // TODO: create function findQuadrantIndex
                let quadrant = board_line[quadrant_index][quadrant_column_index];
                let quadrant_line_str = quadrant.iter().fold("".to_string(),|acc, value| {
                    let value = match value {
                        Some(match_value) => match_value.to_string(),
                        None => "?".to_string()
                        
                    };
                    format!("{acc} {value}")
                });

                line_str.push_str(&quadrant_line_str);
                column += 3;
            }

            output.push_str(&line_str);
            output.push_str("\n");
            line += 1;
        }

        write!(f, "{}", output)
    }
}
