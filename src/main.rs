use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{env, fs, io, mem};

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Provide path to process");
        return;
    }
    let script_name: &str = &args[0];

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
            .chain(Some(Ok(String::from("[")))) // TODO: use some flag value
            .scan(Vec::new(), |v, l| {
                match v.last() {
                    None => {
                        let s: String = l.unwrap();
                        if s.starts_with("[") {
                            // TODO: use regex from args
                            v.push(s);
                            Some(None)
                        } else {
                            Some(None)
                        }
                    }
                    Some(_) => {
                        let s: String = l.unwrap();
                        if s.starts_with("[") {
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
            .filter(|s| {
                // TODO: filter empty strings
                match s {
                    None => false,
                    Some(_) => true,
                }
            })
            .flatten();

        logs_iterators.push(logs);
    }

    let mut current_logs: Vec<Vec<String>> = vec![];
    let mut current_timestamps: Vec<String> = vec![];

    for it in logs_iterators.iter_mut() {
        match it.next() {
            None => {
                continue;
            }
            Some(log) => {
                // println!("{:?}", &log);
                let timestamp = log.first().unwrap().split("]").next().unwrap().to_string();
                current_timestamps.push(timestamp);
                current_logs.push(log);
            }
        }
    }

    while !logs_iterators.is_empty() {
        let max_val = current_timestamps.iter().min().unwrap();
        let max_i = current_timestamps
            .iter()
            .position(|s| s == max_val)
            .unwrap();

        // dbg!(&current_timestamps);
        // dbg!(&current_logs);

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
                let timestamp = log.first().unwrap().split("]").next().unwrap().to_string();
                current_timestamps[max_i] = timestamp;
                current_logs[max_i] = log;
            }
        }
    }
    dbg!(&script_name);
}

// fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
//     where P: AsRef<Path>, {
//     let file = File::open(filename)?;
//     Ok(io::BufReader::new(file).lines())
// }
