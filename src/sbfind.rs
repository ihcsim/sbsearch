use chrono::{self, DateTime, Utc};
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub struct Entry {
    pub level: String,
    pub path: String,
    content: String,
    timestamp: DateTime<Utc>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = self.content.clone();
        write!(f, "{}", out)
    }
}

pub fn search(dir: &Path, key: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut entries: Vec<Entry> = Vec::new();
    search_tree(dir, key, &mut entries)?;
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(entries)
}

fn search_tree(dir: &Path, key: &str, v: &mut Vec<Entry>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            search_tree(&path, key, v)?;
            continue;
        }

        if path.is_file() {
            search_file(&path, v, key)?;
            continue;
        }

        println!("skipping {}", path.display())
    }
    Ok(())
}

fn search_file(path: &Path, v: &mut Vec<Entry>, s: &str) -> Result<(), Box<dyn Error>> {
    let regex_dt = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z")?;
    let regex_lv = Regex::new(r"level=([^\s]+)")?;

    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line
                && content.contains(s)
            {
                let timestamp = regex_dt.find(content.as_str()).unwrap();
                let timestamp_fixed_offset =
                    DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap();
                let level = match regex_lv.find(content.as_str()) {
                    None => "unknown",
                    Some(r) => r.as_str(),
                };

                let entry = Entry {
                    content: content.clone(),
                    level: String::from(level),
                    path: String::from(path.to_str().unwrap()),
                    timestamp: timestamp_fixed_offset.with_timezone(&Utc),
                };
                v.push(entry);
            }
        }
    }
    Ok(())
}
