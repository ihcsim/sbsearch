use std::fs;
use std::io;
use std::path::Path;

fn main() {
    let path = Path::new("./testdata/supportbundle_e4e6d62c-f3b9-4300-8426-1d8493b2b576_2025-10-27T18-38-27Z");
    if path.is_dir() {
        println!("this is a dir");
        visit_dir(path);
    }
}

fn visit_dir(dir: &Path) -> io::Result<()>{
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path);
        } else {
            println!("found file: {}", entry.path().display());
        }
    }
    Ok(())
}
