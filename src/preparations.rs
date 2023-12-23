use std::fs;
use std::path::PathBuf;

use glob::{Pattern, PatternError};
use regex::{escape, Regex};

pub fn get_valid_strftime(raw_strftime: &Option<String>) -> String {
    match raw_strftime {
        Some(strftime) => {
            println!("Provided strftime: {strftime}");
            strftime.clone()
        }
        None => {
            println!("Use strftime from last completed run");
            String::from("[%F %T,%3f]")
        }
    }
}

pub fn get_valid_re_time(raw_re_time: &Option<String>) -> Result<Regex, regex::Error> {
    match raw_re_time {
        Some(re_time_str) => {
            println!("Provided time regexp: {re_time_str}");
            Regex::new(escape(&re_time_str).as_str())
        }
        None => {
            println!("Use time regexp from config");
            Regex::new(r"^\[[\d\- :,]+\]")
        }
    }
}

pub fn get_valid_dir(raw_dir: &PathBuf) -> Result<PathBuf, String> {
    if !raw_dir.exists() {
        return Err(String::from("Path must exist"));
    }

    if !raw_dir.is_dir() {
        return Err(String::from("Path must be a folder"));
    }

    fs::canonicalize(raw_dir).map_err(|err| err.to_string())
}


pub fn get_valid_glob_filter(raw_glob: &String) -> Result<Pattern, PatternError> {
    Pattern::new(raw_glob)
}

pub fn get_valid_paths(valid_dir: &PathBuf, filter: &Pattern) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(&valid_dir)
        .map_err(|err| format!("Error iterating directory: {}", err))?;

    let paths: Result<Vec<PathBuf>, String> = entries
        .filter_map(|entry_result| {
            let entry = entry_result.map_err(|err| format!("Error reading entry: {}", err)).ok()?;
            let path = entry.path();
            if path.is_dir() {
                None
            } else if filter.matches_path(&path) {
                Some(Ok(path))
            } else {
                None
            }
        })
        .collect();

    paths
}

