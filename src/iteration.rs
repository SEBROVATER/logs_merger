use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::mem;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use log::{error, info, warn};
use regex::Regex;
use tempfile::tempdir;

pub fn get_logs_iterators<'a>(
    logs_paths: impl IntoIterator<Item = &'a PathBuf>,
    re_time: &'a Regex,
) -> Result<Vec<impl Iterator<Item = Vec<String>> + 'a>, String> {
    let mut logs_iterators = vec![];

    for path in logs_paths {
        let file = File::open(&path).map_err(|err| format!("Can't open file: {err}"))?;
        let lines = BufReader::new(file).lines();

        let mut logs = lines
            .chain(Some(Ok(String::from("[end]"))))
            .scan(Vec::new(), |v, line| {
                let line = line.unwrap();

                match v.last() {
                    None => {
                        if re_time.is_match(&line) {
                            v.push(line);
                            Some(None)
                        } else {
                            Some(None)
                        }
                    }
                    Some(_) => {
                        if line == "[end]" || re_time.is_match(&line) {
                            Some(Some(mem::replace(v, vec![line])))
                        } else {
                            v.push(line);
                            Some(None)
                        }
                    }
                }
            })
            .flatten()
            .peekable();

        if let Some(_log) = logs.peek() {
            logs_iterators.push(logs);
        } else {
            warn!("Can't find logs in file {:?}", &path)
        }
    }

    Ok(logs_iterators)
}

pub fn prepare_currents(
    logs_iterators: &mut [impl Iterator<Item = Vec<String>>],
    re_time: &Regex,
    strftime: &str,
) -> Result<(Vec<Vec<String>>, Vec<i64>), String> {
    let mut current_logs: Vec<Vec<String>> = Vec::with_capacity(logs_iterators.len());
    let mut current_timestamps: Vec<i64> = Vec::with_capacity(logs_iterators.len());

    for it in logs_iterators {
        if let Some(log) = it.next() {
            if let Some(first_line) = log.first() {
                if let Some(time_match) = re_time.find(first_line) {
                    let timestamp = NaiveDateTime::parse_from_str(&time_match.as_str(), strftime)
                        .map_err(|err| format!("Failed to parse timestamp: {}", err))?
                        .and_utc().timestamp_millis();

                    current_timestamps.push(timestamp);
                    current_logs.push(log);
                } else {
                    return Err("Can't find 'time' in first line of log".to_string());
                }
            } else {
                warn!("Can't find logs in some file. Skip it");
            }
        } else {
            // Handle the case where there are no more logs in the iterator
            // You might want to log or handle this scenario appropriately
        }
    }

    Ok((current_logs, current_timestamps))
}

pub fn write_to_file(
    output_path: &PathBuf,
    logs_paths: &Vec<PathBuf>,
    re_time: &Regex,
    strftime: &str,
) {
    let mut logs_iterators =
        get_logs_iterators(logs_paths, re_time).expect("Can't prepare iterators over files");

    let (mut current_logs, mut current_timestamps) =
        match prepare_currents(&mut logs_iterators, &re_time, &strftime) {
            Ok(currents) => currents,
            Err(err) => {
                error!("{err}");
                return;
            }
        };

    let file = File::create(output_path).expect("Can't create file to write");
    let mut file = LineWriter::new(file);

    while !logs_iterators.is_empty() {
        let max_val = current_timestamps.iter().min().unwrap();
        let max_i = current_timestamps
            .iter()
            .position(|s| s == max_val)
            .unwrap();

        let max_log = current_logs.get(max_i).unwrap();
        info!("{:?}", &max_log.join("\n"));
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
                let _ = logs_iterators.remove(max_i);
                current_logs.remove(max_i);
                current_timestamps.remove(max_i);
            }
            Some(log) => {
                let timestamp = NaiveDateTime::parse_from_str(
                    re_time.find(&log.first().unwrap()).unwrap().as_str(),
                    &strftime,
                )
                .unwrap()
                .and_utc().timestamp_millis();
                current_timestamps[max_i] = timestamp;
                current_logs[max_i] = log;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::io::Read;
    use crate::logger::set_logger;
    use super::*;
    #[test]
    fn test_multi_single_empty() {
        set_logger(1);
        let tmpdir = tempdir().unwrap();
        // multiline
        let path1 = tmpdir.path().join("file1.log");
        let mut file1 = File::create(&path1).unwrap();
        file1.write_all(b"[01] log1\n[03] log3").unwrap();
        // single line
        let path2 = tmpdir.path().join("file2.log");
        let mut file2 = File::create(&path2).unwrap();
        file2.write_all(b"[02] log2").unwrap();
        // empty file
        let path3 = tmpdir.path().join("file3.log");
        File::create(&path3).unwrap();

        let logs_paths: Vec<PathBuf> = vec![path1, path2, path3];
        let output_path = tmpdir.path().join("merged.log");

        write_to_file(&output_path, &logs_paths, &Regex::new(r"^\[\d+]").unwrap(), &String::from("[%s]"));

        let mut output_string = String::new();
        File::open(&output_path).unwrap().read_to_string(&mut output_string).unwrap();

        assert_eq!(&output_string, "[01] log1\n[02] log2\n[03] log3\n");
    }

}