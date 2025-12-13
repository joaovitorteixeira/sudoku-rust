use std::{
    fs::read_to_string,
    sync::mpsc,
    thread::{self},
};

use crate::{cli::game_updater::GameUpdater, sudoku::algorithms::backtracking::Backtracking};

mod cli;
mod sudoku;

fn main() {
    let (board_tx, board_rx) = mpsc::channel::<String>();
    let board_file_result = read_file("input.txt".to_owned());
    let board_file = match board_file_result {
        Ok(board_file) => board_file,
        Err(msg) => panic!("{}", msg),
    };
    let board = sudoku::board::SudokuBoard::new(board_file, board_tx);

    match board {
        Ok(mut board) => {
            let game_updater = GameUpdater::new(board_rx);
            thread::spawn(|| game_updater.listen());

            let _ = thread::spawn(move || {
                let backtracking = Backtracking::new(&mut board);
                backtracking.resolve();
            })
            .join();
        }
        Err(message) => panic!("{message}"),
    }
}

fn read_file(file_path: String) -> Result<Vec<Vec<Option<u8>>>, String> {
    let mut board_file: Vec<Vec<Option<u8>>> = Vec::new();
    for line in match read_to_string(file_path) {
        Ok(file) => file,
        Err(_) => return Err("Couldn't read the file".to_string()),
    }
    .lines()
    {
        let list = line
            .chars()
            .fold(Vec::<Option<u8>>::new(), |mut acc, value| {
                acc.push(value.to_digit(10).map(|digit| digit as u8));
                acc
            });

        board_file.push(list);
    }

    Ok(board_file)
}
