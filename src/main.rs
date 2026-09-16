use std::env;
use std::fs;
use std::process::ExitCode;

use gridnorm::{format_grid, normalize};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: gridnorm <path-to-grid-file>");
            return ExitCode::FAILURE;
        }
    };

    let input = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not read {}: {}", path, err);
            return ExitCode::FAILURE;
        }
    };

    match normalize(&input) {
        Ok(grid) => {
            println!("{}", format_grid(&grid));
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
