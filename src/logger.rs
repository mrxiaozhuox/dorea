use std::{fs, path::PathBuf};


pub fn init_logger(path: String, quiet: bool) {

    let root = PathBuf::from(&path).join("log");

    println!("{} = {:?}",path, root);
    if !root.is_dir() {
        fs::create_dir(root).unwrap();
    }

    // reqwest::get("")
}