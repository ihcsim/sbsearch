use clap::Parser;
use std::error::Error;

mod sbsearch;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let keyword = args.keyword.as_str();
    let root_dir = format!("{}/{}", args.support_bundle_path, "logs");
    let root_dir = root_dir.as_str();
    let mut terminal = ratatui::init();
    tui::Tui::new(root_dir, keyword).run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    support_bundle_path: String,

    #[arg(short, long)]
    keyword: String,
}
