use chrono::{self, DateTime, Utc};
use regex::Regex;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let root_dir =
        "./testdata/supportbundle_e4e6d62c-f3b9-4300-8426-1d8493b2b576_2025-10-27T18-38-27Z/logs";
    let key = String::from("pvc-00b250c3-3e44-4cc8-a9c8-532621b4b1ea");

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
    let root = Path::new(ROOT_DIR);
    let regex_dt = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z").unwrap();
    let regex_lv = Regex::new(r"level=([^\s]+)").unwrap();

    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line
                && content.contains(s)
            {
                let subpath = path.strip_prefix(root).unwrap();
                let identifier: Vec<&str> = subpath.to_str().unwrap().split('/').collect();
                let timestamp = regex_dt.find(content.as_str()).unwrap();
                let timestamp_fixed_offset = DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap();
                let level = match regex_lv.find(content.as_str()) {
                    None => "unknown",
                    Some(r) => r.as_str(),
                };

                let entry = Entry {
                    component: Component {
                        name: String::from(identifier[1]),
                        namespace: String::from(identifier[0]),
                    },
                    content: content.clone(),
                    level: String::from(level),
                    path: String::from(subpath.to_str().unwrap()),
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
        write!(f, "{}", self.content)
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
