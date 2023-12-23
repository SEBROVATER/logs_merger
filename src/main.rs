use std::{fs, io, mem};
use std::fs::File;
use std::io::{BufRead, LineWriter, Write};

use chrono::NaiveDateTime;
use clap::Parser;

use cli::Cli;
use preparations::{get_valid_dir, get_valid_glob_filter, get_valid_re_time, get_valid_strftime};
use strings_similarity::get_common_substring;

use crate::preparations::{get_valid_dir, get_valid_glob_filter, get_valid_re_time, get_valid_strftime};

mod cli;
mod strings_similarity;
mod preparations;

fn main() {
    let cli = Cli::parse();

    let strftime = get_valid_strftime(&cli.strftime);

    let re_time = match get_valid_re_time(&cli.re_time){
        Ok(re) => re,
        Err(err) => {
            println!("Can't compile regexps for time parsing: {err}");
            return;
        }
    };

    let logs_dir = match get_valid_dir(&cli.dir) {
        Ok(dir) => dir,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let filter = match get_valid_glob_filter(&cli.filter) {
        Ok(glob_pattern) => glob_pattern,
        Err(_) => {
            println!("Can't compile glob pattern");
            return;
        }
    };

    let logs_paths: Vec<PathBuf> = fs::read_dir(&logs_dir)
        .expect("Can't iterate over dir")
        .filter(|path| match path {
            Err(_) => {
                panic!("Can't iterate over dir");
            }
            Ok(path) => !&path.path().is_dir() && filter.matches_path(&path.path()),
        })
        .map(|dir_entry| dir_entry.unwrap().path())
        .collect();

    let file_name = match cli.output {
        None => {
            let first_path = match logs_paths.iter().peekable().next() {
                None => return,
                Some(path) => path,
            };

            logs_paths
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap())
                .fold(
                    first_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    |path1, path2| get_common_substring(path1.as_str(), path2),
                )
                .trim_start_matches('_')
                .to_string()
        }
        Some(file_name) => {
            if file_name.starts_with("./") || file_name.starts_with("/") {
                println!("Don't use paths for output name");
                return;
            };
            file_name
        }
    };
    dbg!(&file_name);
    if !logs_dir.join("merged").exists() {
        fs::create_dir(logs_dir.join("merged")).expect("Can't create 'merged' dir");
    }

    let mut logs_iterators = vec![];
    for path in logs_paths.iter() {
        dbg!(&path);

        let file = File::open(&path).unwrap();
        let lines = io::BufReader::new(file).lines();

        let logs = lines
            .chain(Some(Ok(String::from("[end]")))) // TODO: use some flag value
            .scan(Vec::new(), |v, l| {
                match v.last() {
                    None => {
                        let s: String = l.unwrap();
                        if re_time.is_match(&s) {
                            // TODO: use regex from args

                            v.push(s);
                            Some(None)
                        } else {
                            Some(None)
                        }
                    }
                    Some(_) => {
                        let s: String = l.unwrap();
                        if s == String::from("[end]") || re_time.is_match(&s) {
                            // TODO: use regex from args

                            // println!("{:?}", v);
                            Some(Some(mem::replace(v, vec![s])))
                        } else {
                            v.push(s);
                            Some(None)
                        }
                    }
                }
            })
            .flatten();

        logs_iterators.push(logs);
    }

    let output_path = logs_dir.join("merged").join(file_name);

    let file = File::create(output_path).expect("Can't create file to write");
    let mut file = LineWriter::new(file);

    let mut current_logs: Vec<Vec<String>> = vec![];
    let mut current_timestamps: Vec<i64> = vec![];

    for it in logs_iterators.iter_mut() {
        let log = it.next().expect("Can't find logs in some file");

        // println!("{:?}", &log);
        let timestamp = NaiveDateTime::parse_from_str(
            re_time.find(&log.first().unwrap()).unwrap().as_str(),
            &strftime,
        )
            .unwrap()
            .timestamp_millis();
        current_timestamps.push(timestamp);
        current_logs.push(log);
    }

    while !logs_iterators.is_empty() {
        // println!("{:?}", &logs_iterators.len());
        // println!("{:?}", &current_timestamps);
        // println!("{:?}", &current_logs);

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
