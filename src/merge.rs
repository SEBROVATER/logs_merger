use log::error;

use crate::cli::MergeCli;
use crate::iteration::write_to_file;
use crate::logger::set_logger;
use crate::preparations;

pub fn merge(cli: MergeCli){
    set_logger(cli.verbose);
    let strftime = preparations::get_valid_strftime(&cli.strftime);

    let re_time = match preparations::get_valid_re_time(&cli.re_time) {
        Ok(re) => re,
        Err(err) => {
            error!("Can't compile regexps for time parsing: {err}");
            return;
        }
    };

    let logs_dir = match preparations::get_valid_dir(&cli.dir) {
        Ok(dir) => dir,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let filter = match preparations::get_valid_glob_filter(&cli.glob) {
        Ok(glob_pattern) => glob_pattern,
        Err(_) => {
            error!("Can't compile glob pattern");
            return;
        }
    };

    let logs_paths = match preparations::get_valid_paths(&logs_dir, &filter) {
        Ok(paths) => paths,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let file_name = match preparations::get_valid_output_name(&cli.output, &logs_paths) {
        Ok(name) => name,
        Err(err) => {
            error!("{err}");
            return;
        }
    };
    let output_path = logs_dir.join(file_name);

    write_to_file(&output_path, &logs_paths, &re_time, &strftime);
}