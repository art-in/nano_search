use anyhow::{Result, bail};
use nano_search::commands;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        bail!("no command specified");
    }

    tracing_subscriber::fmt::init();

    match args[1].as_str() {
        "--index" => commands::index_command()?,
        "--eval" => commands::eval_command()?,
        "--search" => commands::search_command()?,
        _ => {
            print_usage();
            bail!("unknown command: {}", args[1]);
        }
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: nano_search [--index | --eval | --search]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  --index   Index documents using all search engines");
    eprintln!("  --eval    Evaluate search quality for all engines");
    eprintln!("  --search  Perform a search test with a single query");
    eprintln!();
}
