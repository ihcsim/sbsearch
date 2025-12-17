use std::fs::{self,DirEntry};
use std::io;
use std::path::Path;

fn main() -> io::Result<()>{
    let path = Path::new("./testdata/supportbundle_e4e6d62c-f3b9-4300-8426-1d8493b2b576_2025-10-27T18-38-27Z");
    visit_dir(path, &search)?;
    Ok(())
}

fn visit_dir(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()>{
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, cb)?;
        } else {
            search(&entry)
        }
    }
    Ok(())
}

fn search(entry: &DirEntry) {
    println!(">> {:?}", entry.path())
}
