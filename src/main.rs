use std::env;
use std::fs;
use std::process::ExitCode;

use gridnorm::{format_grid, normalize};

fn main() -> ExitCode {
    let mut check_symmetry = false;
    let mut path = None;
    for arg in env::args().skip(1) {
        if arg == "--check-symmetry" {
            check_symmetry = true;
        } else if path.is_none() {
            path = Some(arg);
        } else {
            eprintln!("unexpected argument: {}", arg);
            return ExitCode::FAILURE;
        }
    }

    let path = match path {
        Some(path) => path,
        None => {
            eprintln!("usage: gridnorm [--check-symmetry] <path-to-grid-file>");
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
            if check_symmetry {
                let violations = grid.symmetry_violations();
                if !violations.is_empty() {
                    for ((r1, c1), (r2, c2)) in &violations {
                        eprintln!(
                            "not symmetric: row {} col {} does not match row {} col {}",
                            r1, c1, r2, c2
                        );
                    }
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
