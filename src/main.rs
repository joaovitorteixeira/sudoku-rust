pub mod sudoku;

fn main() {
    let mut columns: Vec<&[Option<u8>]> = Vec::new();
    let mut index = 0;

    while index < 9 {
        columns.push(&[None; 9]);
        index += 1;
    }

    let board = sudoku::board::SudokuBoard::new(columns);

    match board {
        Ok(board) => println!("{:}", board),
        Err(message) => panic!("{message}"),
    }
}
