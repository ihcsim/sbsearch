use clap::Parser;
use std::error::Error;
use std::path::Path;

mod sbfind;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let resource_name = args.resource_name.as_str();
    let root_dir = args.support_bundle_path + "/logs";
    let root_path = Path::new(root_dir.as_str());

    let entries = sbfind::search(root_path, resource_name)?;

    let mut terminal = ratatui::init();
    tui::new(root_dir, entries).run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    support_bundle_path: String,

    #[arg(short, long)]
    resource_name: String,
}
