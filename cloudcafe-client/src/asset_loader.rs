use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn load_assets() {
    let _ = std::fs::create_dir("assets");
    let mut open_options = OpenOptions::new();
    open_options.write(true).create(true);
    if File::open("assets\\cubemap.hdr").is_err() {
        let mut file = open_options.open("assets\\cubemap.hdr").unwrap();
        file.write_all(include_bytes!("..\\assets\\cubemap.hdr")).unwrap();
    }
}