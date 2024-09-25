use clap::builder::ArgAction;
use clap::{Args, Parser, Subcommand};
use glob::glob;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct RenameArgs {
    pub path: std::path::PathBuf,
    #[arg(short = 'f', long = "filter")]
    pub filter_string: String,
    #[arg(short = 'p', long = "pattern")]
    pub pattern: String,
    #[arg(short = 's', long = "substitute")]
    pub substitute: String,
    #[arg(short = 'r', long = "recursive", action=ArgAction::SetTrue)]
    pub recursive: bool,
}

pub fn get_files(dir: &Path, glob_pattern: &str, recursive: bool) {
    let full_glob_pattern = if recursive == true {
        PathBuf::from(dir).join("**")
    } else {
        PathBuf::from(dir)
    };

    let full_glob_pattern = full_glob_pattern.join(glob_pattern);

    println!("full glob: {}", full_glob_pattern.to_str().unwrap());

    let mut files: Vec<PathBuf> = Vec::new();

    for entry in glob(full_glob_pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(file_path) => {
                println!("{:?}", file_path.display());
                if file_path.is_file() {
                    files.push(file_path.clone())
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    println!("Files only:");
    for i in files {
        println!("{}", i.to_str().unwrap())
    }
}

pub fn rename(
    path: &std::path::PathBuf,
    filter_string: &str,
    pattern: &str,
    substitute: &str,
    recursive: bool,
) {
    // get file to rename
    get_files(path, filter_string, recursive);
}
