use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[command(long_about = "Merges log fields by its timestamps")]
pub struct Cli {
    /// Directory with log files
    pub dir: PathBuf,

    /// Name for generated file
    #[arg(long, short, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Regular expression to find timestamps and detect multiline logs
    ///
    /// Example: "^\[\d{1,4}[\d/:, ]+\d{1,3}\]"
    #[arg(long, value_name = "REGEXP")]
    pub re_time: Option<String>,

    /// strftime pattern to parse timestamps found by --re-time
    ///
    /// Example: "[%D %T,%3f]"
    #[arg(long, value_name = "STRFTIME")]
    pub strftime: Option<String>,

    /// Regular expression to filter log files in dir
    #[arg(
    short,
    long,
    required = false,
    value_name = "GLOB",
    default_value = "*"
    )]
    pub filter: String,
}