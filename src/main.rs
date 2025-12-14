use std::{
    fs::read_to_string,
    sync::mpsc,
    thread::{self},
};

use crate::{
    cli::game_updater::{CliChannelEvent, GameUpdater},
    sudoku::algorithms::{
        backtracking::Backtracking, base_algorithms::BaseAlgorithms,
        candidate_election::CandidateElection,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Algorithms {
    Backtracking,
    CandidateElection,
}

mod cli;
mod sudoku;

fn main() {
    let (board_tx, board_rx) = mpsc::channel::<CliChannelEvent>();
    let (throttle_ms, algorithm) = read_args();
    let board_file_result = read_file("input.txt".to_owned());
    let board_file = match board_file_result {
        Ok(board_file) => board_file,
        Err(msg) => panic!("{}", msg),
    };
    let board = sudoku::board::SudokuBoard::new(board_file, board_tx.clone());
    let mut game_updater = GameUpdater::new(board_rx, throttle_ms);
    let game_updater_thread = thread::spawn(move || {
        let _ = game_updater.listen();
    });

    match board {
        Ok(mut board) => {
            let alg = algorithm.unwrap_or(Algorithms::CandidateElection);

            let _ = thread::spawn(move || match alg {
                Algorithms::Backtracking => {
                    let backtracking = Backtracking::new(&mut board);
                    backtracking.resolve();
                }
                Algorithms::CandidateElection => {
                    let candidate = CandidateElection::new(&mut board);
                    candidate.resolve();
                }
            })
            .join();
        }
        Err(message) => panic!("{message}"),
    }

    let _ = board_tx.send(CliChannelEvent::ForceLastPrint);
    let _ = game_updater_thread.join();
}

fn read_args() -> (Option<u64>, Option<Algorithms>) {
    let mut throttle_ms: Option<u64> = None;
    let mut algorithm: Option<Algorithms> = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--throttle-ms" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<u64>() {
                        throttle_ms = Some(v);
                    }
                }
            }
            "--algorithm" | "-a" => {
                if let Some(val) = args.next() {
                    match val.to_lowercase().as_str() {
                        "backtracking" | "bt" => algorithm = Some(Algorithms::Backtracking),
                        "candidate" | "candidateelection" | "ce" => {
                            algorithm = Some(Algorithms::CandidateElection)
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    (throttle_ms, algorithm)
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
