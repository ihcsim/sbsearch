use chrono::{self, DateTime, Utc};
use clap::Parser;
use colored::*;
use regex::Regex;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let args = Args::parse();
    let root_dir = args.root_dir.as_str();
    let key = args.key;

    let root_path = Path::new(root_dir);
    let mut entries: Vec<Entry> = Vec::new();
    search_tree(root_path, &key, &mut entries, &search).unwrap();

    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    for entry in &entries {
        println!("{}", entry);
    }
}

fn search_tree(
    dir: &Path,
    key: &str,
    v: &mut Vec<Entry>,
    callback: &dyn Fn(&Path, &mut Vec<Entry>, &str),
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            search_tree(&path, key, v, callback)?;
            continue;
        }

        if path.is_file() {
            callback(&path, v, key);
            continue;
        }

        println!("skipping {}", path.display())
    }
    Ok(())
}

fn search(path: &Path, v: &mut Vec<Entry>, s: &str) {
    let regex_dt = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z").unwrap();
    let regex_lv = Regex::new(r"level=([^\s]+)").unwrap();

    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line
                && content.contains(s)
            {
                let identifier: Vec<&str> = path.to_str().unwrap().split('/').collect();
                let timestamp = regex_dt.find(content.as_str()).unwrap();
                let timestamp_fixed_offset = DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap();
                let level = match regex_lv.find(content.as_str()) {
                    None => "unknown",
                    Some(r) => r.as_str(),
                };

                let entry = Entry {
                    component: Component {
                        name: String::from(identifier[5]),
                        namespace: String::from(identifier[4]),
                    },
                    content: content.clone(),
                    level: String::from(level),
                    path: String::from(path.to_str().unwrap()),
                    timestamp: timestamp_fixed_offset.with_timezone(&Utc),
                };

                v.push(entry);
            }
        }
    } else {
        println!("could no open file: {}", path.display());
    }
}

#[derive(Debug)]
struct Entry {
    component: Component,
    content: String,
    level: String,
    path: String,
    timestamp: DateTime<Utc>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.level.as_str() {
            "error" => self.content.red(),
            "warn" => self.content.yellow(),
            "info" => self.content.green(),
            "debug" => self.content.cyan(),
            _ => self.content.blue(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
struct Component {
    name: String,
    namespace: String,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    root_dir: String,

    #[arg(short, long)]
    key: String,
}
