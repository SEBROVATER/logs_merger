use std::{fs, io, mem};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, LineWriter, Write};
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::Parser;
use glob::Pattern;
use regex::{escape, Regex};

fn get_common_substring(str1: &str, str2: &str) -> String {
    let first_sequence: Vec<char> = str1.chars().collect();
    let second_sequence: Vec<char> = str2.chars().collect();

    let mut best_i: usize = 0;
    let mut best_j: usize = 0;
    let mut best_size: usize = 0;

    let mut second_sequence_elements = HashMap::new();
    for (i, item) in second_sequence.iter().enumerate() {
        let counter = second_sequence_elements
            .entry(item)
            .or_insert_with(Vec::new);
        counter.push(i);
    }

    let mut j2len: HashMap<usize, usize> = HashMap::new();
    for (i, item) in first_sequence
        .iter()
        .enumerate()
    {
        let mut new_j2len: HashMap<usize, usize> = HashMap::new();
        if let Some(indexes) = second_sequence_elements.get(item) {
            for j in indexes {
                let j = *j;
                let mut size = 0;
                if j > 0 {
                    if let Some(k) = j2len.get(&(j - 1)) {
                        size = *k;
                    }
                }
                size += 1;
                new_j2len.insert(j, size);
                if size > best_size {
                    best_i = i + 1 - size;
                    best_j = j + 1 - size;
                    best_size = size;
                }
            }
        }
        j2len = new_j2len;
    }

    for _ in 0..2 {
        while best_i > 0 && best_j > 0 &&
            first_sequence.get(best_i - 1) == second_sequence.get(best_j - 1)
        {
            best_i -= 1;
            best_j -= 1;
            best_size += 1;
        }
        while best_i + best_size < first_sequence.len()
            && best_j + best_size < second_sequence.len()
            && first_sequence.get(best_i + best_size) == second_sequence.get(best_j + best_size)
        {
            best_size += 1;
        }
    }

    let res = String::from_iter(&first_sequence[best_i..(best_i + best_size)]);
    res
}


#[derive(Parser)]
#[command(author, version, about)]
#[command(long_about = "Merges log fields by its timestamps")]
struct Cli {
    /// Directory with log files
    dir: PathBuf,

    /// Name for generated file
    #[arg(long, short, value_name = "OUTPUT")]
    output: Option<String>,

    /// Regular expression to find timestamps and detect multiline logs
    ///
    /// Example: "^\[\d{1,4}[\d/:, ]+\d{1,3}\]"
    #[arg(long, value_name = "REGEXP")]
    re_time: Option<String>,

    /// strftime pattern to parse timestamps found by --re-time
    ///
    /// Example: "[%D %T,%3f]"
    #[arg(long, value_name = "STRFTIME")]
    strftime: Option<String>,

    /// Regular expression to filter log files in dir
    #[arg(short, long, required = false, value_name = "GLOB", default_value = "*")]
    filter: String,

}

fn main() {
    let cli = Cli::parse();
    let logs_path = cli.dir;

    let strftime = match cli.strftime {
        Some(strftime) => {
            println!("Provided strftime: {strftime}");
            strftime
        }
        None => {
            println!("Use strftime from last completed run");
            String::from("[%D %T,%3f]")
        }
    };

    let re_time = match cli.re_time {
        Some(re_time_str) => {
            println!("Provided time regexp: {re_time_str}");
            Regex::new(escape(&re_time_str).as_str()).expect("Your previous regex can't be compiled")
        }
        None => {
            println!("Use time regexp from last completed run");
            Regex::new(r"^\[\d{1,4}[\d/:, ]+\d{1,3}]").expect("Your regex can't be compiled")
        }
    };


    if !logs_path.exists() {
        println!("Provided path doesn't exists");
        return;
    }

    if !logs_path.is_dir() {
        println!("Provided path must exist");
        return;
    }

    let logs_path = fs::canonicalize(logs_path).expect("Can't absolutize path");




    let filter = Pattern::new(&cli.filter).expect("Invalid filter glob pattern");

    let logs_paths: Vec<PathBuf> = fs::read_dir(&logs_path)
        .expect("Can't iterate over dir")
        .filter(|path| {
            match path {
                Err(_) => {
                    panic!("Can't iterate over dir");
                }
                Ok(path) => { filter.matches_path(&path.path()) }
            }
        }).map(|dir_entry| dir_entry.unwrap().path()).collect();


    let file_name = match cli.output {
        None => {
            let first_path = match logs_paths.iter().peekable().next() {
                None => return,
                Some(path) => path,
            };

            logs_paths.iter().map(|path| {
                path
                    .file_name().unwrap()
                    .to_str().unwrap()
            })
                .fold(first_path.file_name().unwrap().to_str().unwrap().to_string(),
                      |path1, path2| get_common_substring(path1.as_str(), path2))
                .trim_start_matches('_').to_string()
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
    if !logs_path.join("merged").exists() {
        fs::create_dir(logs_path.join("merged")).expect("Can't create 'merged' dir");
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

    let output_path = logs_path.join("merged").join(file_name);

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
                continue
            };
            file.write_all(log.as_bytes()).expect("Can't write line to file");
            file.write_all(b"\n").expect("Can't write line to file");
        };

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
