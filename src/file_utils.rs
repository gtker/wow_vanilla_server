use std::io::Write;
use std::path::Path;

pub fn append_string_to_file(s: &str, filename: &Path) {
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(filename)
        .unwrap();
    f.write_all(s.as_bytes()).unwrap();
}
