use anyhow::Result;
use clap::Parser;
use nano_search::cli::args::{Args, Command};
use nano_search::cli::commands;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Command::Index { threads } => {
            commands::index(&args.engines, &args.dataset, threads)?;
        }
        Command::Eval => commands::eval(&args.engines, &args.dataset)?,
        Command::Search => commands::search(&args.engines, &args.dataset)?,
    }

    Ok(())
}
