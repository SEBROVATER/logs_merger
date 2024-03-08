use clap::Parser;

use cli::Cli;

use crate::cli::Commands;
use crate::merge::merge;

mod cli;
mod iteration;
mod logger;
mod merge;
mod preparations;
mod strings_similarity;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Merge(cli) => {
            merge(cli);
        }
    }
}
