use std::fs::File;
use std::io::{LineWriter, Write};

use chrono::NaiveDateTime;
use clap::Parser;

use cli::Cli;

use crate::iteration::prepare_currents;

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

    let mut logs_iterators = iteration::get_logs_iterators(&logs_paths, &re_time).expect("Can't prepare iterators over files");

    let (mut current_logs, mut current_timestamps) = match prepare_currents(&mut logs_iterators, &re_time, &strftime) {
        Ok(currents) => currents,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let output_path = logs_dir.join(file_name);
    let file = File::create(output_path).expect("Can't create file to write");
    let mut file = LineWriter::new(file);


    while !logs_iterators.is_empty() {

        let max_val = current_timestamps.iter().min().unwrap();
        let max_i = current_timestamps
            .iter()
            .position(|s| s == max_val)
            .unwrap();

        let max_log = current_logs.get(max_i).unwrap();
        println!("{:?}", &max_log);
        for log in max_log.iter() {
            if log.is_empty() {
                continue;
            };
            file.write_all(log.as_bytes())
                .expect("Can't write line to file");
            file.write_all(b"\n").expect("Can't write line to file");
        }

        let it = logs_iterators.get_mut(max_i).unwrap();
        match it.next() {
            None => {
                current_logs.remove(max_i);
                current_timestamps.remove(max_i);
                let _ = logs_iterators.remove(max_i);
            }
            Some(log) => {
                let timestamp = NaiveDateTime::parse_from_str(
                    re_time.find(&log.first().unwrap()).unwrap().as_str(),
                    &strftime,
                )
                    .unwrap()
                    .timestamp_millis();
                current_timestamps[max_i] = timestamp;
                current_logs[max_i] = log;
            }
        }
    }
}

// fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
//     where P: AsRef<Path>, {
//     let file = File::open(filename)?;
//     Ok(io::BufReader::new(file).lines())
// }
