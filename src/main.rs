use std::{env, fs, io, mem};
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

use chrono::NaiveDateTime;
use regex::Regex;

fn main() {

    let strf_datetime = String::from("[%D %T,%3f]");
    let re_datetime =
        Regex::new(r"^\[\d{1,4}[\d/:, ]+\d{1,3}\]").expect("Your regex can't be compiled");

    // let flag = re_datetime.find("[11/04/23 10:00:00,001] multi1").unwrap().as_str();
    // println!("{:?}", NaiveDateTime::parse_from_str(flag, &strf_datetime));
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Provide path to process");
        return;
    }
    // let script_name: &str = &args[0];

    let logs_path = Path::new(&args[1]);
    if !logs_path.exists() {
        println!("Provided path doesn't exists");
        return;
    }

    if !logs_path.is_dir() {
        println!("Provided path must exist");
        return;
    }
    dbg!(&logs_path);
    let logs_path = fs::canonicalize(logs_path).expect("Can't absolutize path");

    let mut logs_iterators = vec![];

    for dir_entry in fs::read_dir(&logs_path).unwrap() {
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
                        if re_datetime.is_match(&s) {
                            // TODO: use regex from args

                            v.push(s);
                            Some(None)
                        } else {
                            Some(None)
                        }
                    }
                    Some(_) => {
                        let s: String = l.unwrap();
                        if s == String::from("[end]") || re_datetime.is_match(&s) {
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
            re_datetime.find(&log.first().unwrap()).unwrap().as_str(),
            &strf_datetime,
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
                    re_datetime.find(&log.first().unwrap()).unwrap().as_str(),
                    &strf_datetime,
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
