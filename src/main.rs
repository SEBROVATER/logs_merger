use clap::Parser;

use cli::Cli;

use crate::cli::Commands;
use crate::merge::merge;

mod cli;
mod preparations;
mod strings_similarity;
mod iteration;
mod logger;
mod merge;

fn main() {

    let cli = Cli::parse();

    match cli.command {
        Commands::Merge(cli) => {
            merge(cli);

        }
    }


}
