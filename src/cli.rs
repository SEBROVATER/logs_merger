use std::path::PathBuf;

use clap::ArgAction::Count;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,


}

#[derive(Subcommand)]
pub enum Commands {
    ///
    Merge(MergeCli),
}

#[derive(Args)]
#[command(long_about = "Merges log files by its timestamps. \
By default works with .txt and .log extensions")]
pub struct MergeCli {
    /// Directory with log files
    pub dir: PathBuf,

    /// Name for generated file
    #[arg(long, short, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Regular expression to find timestamps and detect start of logs
    ///
    /// Example: r"^\[\d{1,4}[\d/:, ]+\d{1,3}\]"
    #[arg(long, value_name = "REGEXP")]
    pub re_time: Option<String>,

    /// strftime pattern to parse timestamps found by --re-time
    ///
    /// Example: "[%D %T,%3f]"
    #[arg(long, value_name = "STRFTIME")]
    pub strftime: Option<String>,

    /// Regular expression to filter log files in dir
    ///
    /// Can be used to filter specific extensions,
    /// for example "*.log"
    #[arg(
        long,
        // required = false,
        value_name = "GLOB",
        default_value = "*"
    )]
    pub glob: String,

    /// Set verbosity level
    #[arg(
    short,
    long,
    action = Count,
    global = true,
    )]
    pub verbose: u8,

}
