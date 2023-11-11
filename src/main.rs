use std::{fs, io, mem};
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::Parser;
use glob::Pattern;
use regex::{escape, Regex};

#[derive(Parser)]
#[command(author, version, about)]
#[command(long_about = "Merges log fields by its timestamps")]
struct Cli {

    /// Directory with log files
    dir: PathBuf,

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


    if !logs_path.join("merged").exists() {
        fs::create_dir(logs_path.join("merged")).expect("Can't create 'merged' dir");
    }


    let filter = Pattern::new(&cli.filter).expect("Invalid filter glob pattern");

    let logs_paths = fs::read_dir(&logs_path)
        .expect("Can't iterate over dir")
        .filter(| path | {
            match path {
                Err(E) => {
                    panic!("Can't iterate over dir");
                },
                Ok(path) => { filter.matches_path(&path.path()) },
            }
        });

    let mut logs_iterators = vec![];

    for dir_entry in  logs_paths {
        let dir_entry = dir_entry.unwrap();
        let path = dir_entry.path();
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
