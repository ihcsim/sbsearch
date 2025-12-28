use clap::Parser;
use std::error::Error;
use std::path::Path;

mod sbfind;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let key = args.key.as_str();
    let root_dir = args.root_dir;
    let root_path = Path::new(root_dir.as_str());

    let entries = sbfind::search(root_path, key)?;

    let mut terminal = ratatui::init();
    tui::new(root_dir, entries).run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    root_dir: String,

    #[arg(short, long)]
    key: String,
}
