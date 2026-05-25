use anyhow::Result;
use clap::Parser;
use nano_search::cli::args::{Args, Command};
use nano_search::cli::commands;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Command::Index { threads } => {
            commands::index(
                &args.engines,
                &args.dataset,
                &args.parent_index_dir,
                threads,
            )?;
        }
        Command::Eval => commands::eval(
            &args.engines,
            &args.dataset,
            &args.parent_index_dir,
        )?,
        Command::Search => commands::search(
            &args.engines,
            &args.dataset,
            &args.parent_index_dir,
        )?,
    }

    Ok(())
}
