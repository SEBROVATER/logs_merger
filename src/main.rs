use std::fs::File;
use std::io::{LineWriter, Write};

use chrono::NaiveDateTime;
use clap::Parser;

use cli::Cli;

use crate::iteration::{prepare_currents, write_to_file};

mod cli;
mod preparations;
mod strings_similarity;
mod iteration;

fn main() {
    let cli = Cli::parse();

    let strftime = preparations::get_valid_strftime(&cli.strftime);

    let re_time = match preparations::get_valid_re_time(&cli.re_time) {
        Ok(re) => re,
        Err(err) => {
            println!("Can't compile regexps for time parsing: {err}");
            return;
        }
    };

    let logs_dir = match preparations::get_valid_dir(&cli.dir) {
        Ok(dir) => dir,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let filter = match preparations::get_valid_glob_filter(&cli.filter) {
        Ok(glob_pattern) => glob_pattern,
        Err(_) => {
            println!("Can't compile glob pattern");
            return;
        }
    };

    let logs_paths = match preparations::get_valid_paths(&logs_dir, &filter) {
        Ok(paths) => paths,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let file_name = match preparations::get_valid_output_name(&cli.output, &logs_paths) {
        Ok(name) => name,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    let output_path = logs_dir.join(file_name);

    write_to_file(&output_path, &logs_paths, &re_time, &strftime);
}
