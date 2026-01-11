use chrono::{self, DateTime, Utc};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, sinks::UTF8};
use std::error::Error;
use std::fmt;
use std::fs::{self};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Entry {
    pub level: String,
    pub path: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

pub struct SearchResult {
    pub entries_offset: Vec<Entry>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = self.content.clone();
        write!(f, "{}", out)
    }
}

pub fn search(
    dir: &Path,
    key: &str,
    offset: usize,
    limit: usize,
    cache: &mut Vec<Entry>,
) -> Result<SearchResult, Box<dyn Error>> {
    if cache.is_empty() {
        search_tree(dir, key, cache)?;
        cache.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    let limit = limit.min(cache.len().saturating_sub(offset));
    let entries_offset = cache.iter().skip(offset).take(limit).cloned().collect();

    Ok(SearchResult { entries_offset })
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

fn search_file(path: &Path, entries: &mut Vec<Entry>, keyword: &str) -> Result<(), Box<dyn Error>> {
    let matcher = RegexMatcher::new(keyword)?;
    let matcher_timestamp = RegexMatcher::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z")?;
    let matcher_log_level = RegexMatcher::new(r"level=([^\s]+)")?;
    Searcher::new().search_path(
        &matcher,
        path,
        UTF8(|_lnum, line| {
            let timestamp = matcher_timestamp.find(line.as_bytes())?.unwrap();
            let timestamp_fixed_offset = DateTime::parse_from_rfc3339(&line[timestamp]).unwrap();
            let level = match matcher_log_level.find(line.as_bytes()) {
                Ok(opt) => {
                    if let Some(m) = opt {
                        line[m.start()..m.end()].split('=').nth(1).unwrap()
                    } else {
                        "UNKNOWN"
                    }
                }
                Err(_) => "UNKNOWN",
            };
            let entry = Entry {
                content: String::from(line),
                level: String::from(level),
                path: String::from(path.to_str().unwrap()),
                timestamp: timestamp_fixed_offset.with_timezone(&Utc),
            };
            entries.push(entry);
            Ok(true)
        }),
    )?;
    Ok(())
}
