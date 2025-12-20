use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> io::Result<()> {
    let path = Path::new(
        "./testdata/supportbundle_e4e6d62c-f3b9-4300-8426-1d8493b2b576_2025-10-27T18-38-27Z/logs",
    );
    visit_dir(path, &search)?;
    Ok(())
}

fn visit_dir(dir: &Path, search: &dyn Fn(&Path, &str)) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, search)?;
            continue;
        }

        if path.is_file() {
            search(&path, "asbc-ci-int-cicd-backup-data-disk");
            continue;
        }

        println!("skipping {}", path.display())
    }
    Ok(())
}

fn search(path: &Path, s: &str) {
    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line
                && content.contains(s)
            {
                let entry = Entry {
                    component: Component {
                        name: String::from(""),
                        namespace: String::from(""),
                    },
                    content,
                    level: String::from(""),
                    path: String::from(path.to_string_lossy()),
                    timestamp: String::from(""),
                };
                dbg!(&entry);
                println!("{}", entry);
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
    timestamp: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.timestamp, self.path, self.component, self.level, self.content
        )
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
