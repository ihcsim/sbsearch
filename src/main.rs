use clap::Parser;
use sbfind::Entry;
use std::io;
use std::path::Path;

mod sbfind;
mod tui;

fn main() -> io::Result<()> {
    let args = Args::parse();
    let key = args.key.as_str();
    let root_path = Path::new(args.root_dir.as_str());

    let mut entries: Vec<Entry> = Vec::new();
    sbfind::search(root_path, key, &mut entries).unwrap();

    let mut terminal = ratatui::init();
    let app_out = tui::new(entries).run(&mut terminal);
    ratatui::restore();
    app_out
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    root_dir: String,

    #[arg(short, long)]
    key: String,
}
