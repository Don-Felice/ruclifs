use clap::builder::ArgAction;
use clap::{Args, Parser, Subcommand};
use glob::glob;
use regex::Regex;
use std::io;
use std::path::{self, Path, PathBuf};

const LINE: &str = "-------------------";
#[derive(Args, Debug)]
pub struct RenameArgs {
    pub path: std::path::PathBuf,
    #[arg(short = 'f', long = "filter", default_value_t=String::from("*"))]
    pub filter_string: String,
    #[arg(short = 'p', long = "pattern")]
    pub pattern: String,
    #[arg(short = 's', long = "substitute")]
    pub substitute: String,
    #[arg(short = 'r', long = "recursive", action=ArgAction::SetTrue)]
    pub recursive: bool,
    #[arg(short = 'S', long = "skip_preview", action=ArgAction::SetTrue)]
    pub skip_preview: bool,
}

pub fn get_files(dir: &Path, glob_pattern: &str, recursive: bool) -> Vec<PathBuf> {
    let full_glob_pattern = if recursive == true {
        PathBuf::from(dir).join("**")
    } else {
        PathBuf::from(dir)
    };

    let full_glob_pattern = full_glob_pattern.join(glob_pattern);

    let mut files: Vec<PathBuf> = Vec::new();

    for entry in glob(full_glob_pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(file_path) => {
                if file_path.is_file() {
                    files.push(file_path.clone())
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    files
}

fn rename_file(path_file: &PathBuf, pattern: &str, substitute: &str, dry_run: bool) {
    let file_name = path_file.file_name().unwrap().to_str().unwrap();
    let re = Regex::new(pattern).unwrap();
    let file_name_new = re.replace_all(file_name, substitute).to_string();
    println!("{} -> {}", file_name, file_name_new);
    let path_new = path_file.parent().unwrap().join(file_name_new);
    if !dry_run {
        let _ = std::fs::rename(path_file, path_new);
    }
}

fn proceed_query() {
    println!("\nIf you wanna apply this renaming, give me a 'yes' or 'y' now:");
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            if input.trim() != "yes" && input.trim() != "y" {
                println!("Will abort here. See you soon!");
                std::process::exit(0)
            }
        }
        Err(error) => println!("error: {error}"),
    }
}

pub fn rename(
    path: &std::path::PathBuf,
    filter_string: &str,
    pattern: &str,
    substitute: &str,
    recursive: bool,
    skip_preview: bool,
) {
    // get file to rename
    let files = get_files(path, filter_string, recursive);
    println!("Renaming {} files:", files.len());

    if !skip_preview {
        println!("{}", LINE);
        println!("Preview:");
        for file in &files {
            let _ = rename_file(file, pattern, substitute, true);
        }
        proceed_query();
    }
    println!("{}", LINE);
    for file in &files {
        let _ = rename_file(file, pattern, substitute, false);
    }
    println!("{}", LINE);
}
