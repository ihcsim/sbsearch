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
                println!("{}: {}", path.display(), content);
            }
        }
    } else {
        println!("could no open file: {}", path.display());
    }
}
