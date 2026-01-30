use chrono::Local;
use clap::Parser;
use log::*;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

mod sbsearch;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let keyword = args.keyword.as_str();
    let root_dir = args.support_bundle_path.as_str();

    if let Some(l) = args.log_level {
        let log_level = LevelFilter::from_str(l.as_str())?;
        let target = Box::new(File::create(".sbsearch.log")?);
        env_logger::Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(None, log_level)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
    }

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

    #[arg(short, long)]
    log_level: Option<String>,
}
