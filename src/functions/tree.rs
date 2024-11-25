use crate::utils::cli::Styler;
use clap::builder::ArgAction;
use clap::Args;
use std::env::current_dir;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::path::PathBuf;

const PIPE: &str = "│";
const ELBOW: &str = "└── ";
const TEE: &str = "├── ";
const PIPE_PREFIX: &str = "│   ";
const SPACE_PREFIX: &str = "    ";
const SPACE_SIZE: &str = " ";

#[derive(Args, Debug)]
pub struct TreeArgs {
    #[arg(default_value = ".")]
    pub path: std::path::PathBuf,
    #[arg(short = 'd', long = "depth", default_value = "-1")]
    pub depth: i32,
    #[arg(short = 's', long = "hide_size", action=ArgAction::SetTrue)]
    pub hide_size: bool,
}

trait Size {
    fn get_size(&self) -> i32;
    //get size in bites
}

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    prefix: String,
    connector: String,
    show_size: bool,
    size: Option<i32>,
}

#[derive(Clone)]
struct DirEntry {
    path: PathBuf,
    prefix: String,
    connector: String,
    depth: i32,
    th_depth: i32,
    show_size: bool,
    have_access: bool,
    children_file: Vec<FileEntry>,
    children_dir: Vec<DirEntry>,
    size: Option<i32>,
}

impl FileEntry {
    fn build(path: PathBuf, prefix: String, connector: String, show_size: bool) -> FileEntry {
        let size: Option<i32>;
        if show_size {
            size = Some(42);
        } else {
            size = None;
        }
        FileEntry {
            path: path,
            prefix: prefix,
            connector: connector,
            show_size: show_size,
            size: size,
        }
    }
}

impl DirEntry {
    fn build(
        path: PathBuf,
        prefix: String,
        connector: String,
        depth: i32,
        th_depth: i32,
        show_size: bool,
    ) -> DirEntry {
        let size: Option<i32>;
        if show_size {
            size = Some(42)
        } else {
            size = None
        };

        DirEntry {
            path: path,
            prefix: prefix,
            connector: connector,
            depth: depth,
            th_depth: th_depth,
            show_size: show_size,
            have_access: true, //TODO: have logic here
            children_file: Vec::new(),
            children_dir: Vec::new(),
            size: size,
        }
    }
    fn get_children(&mut self) {
        let mut children_file: Vec<FileEntry> = Vec::new();
        let mut children_dir: Vec<DirEntry> = Vec::new();

        if (self.th_depth >= 0) && (self.depth >= self.th_depth) && !self.show_size {
            self.children_file = children_file;
            self.children_dir = children_dir;
            return;
        }

        let mut child_prefix = self.prefix.clone();
        if self.connector == TEE {
            child_prefix.push_str(PIPE_PREFIX);
        } else if self.connector == ELBOW {
            child_prefix.push_str(SPACE_PREFIX);
        }

        let mut content = fs::read_dir(self.path.clone())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect::<Vec<_>>();

        content.sort_by_key(|e| e.is_dir());

        let len_content = content.len();

        for (num_path, path) in content.iter().enumerate() {
            let child_connector: String;
            if num_path + 1 < len_content {
                child_connector = TEE.to_string()
            } else {
                child_connector = ELBOW.to_string()
            }
            if path.is_dir() {
                let mut new_dir_entry = DirEntry::build(
                    path.to_path_buf(),
                    child_prefix.clone(),
                    child_connector,
                    self.depth + 1,
                    self.th_depth,
                    self.show_size,
                );
                new_dir_entry.get_children();
                children_dir.push(new_dir_entry);
            } else {
                children_file.push(FileEntry::build(
                    path.to_path_buf(),
                    child_prefix.clone(),
                    child_connector,
                    self.show_size,
                ));
            }
        }
        self.children_file = children_file;
        self.children_dir = children_dir;
    }
}

impl fmt::Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!(
            "{}{}{}",
            self.prefix,
            self.connector,
            self.path.file_name().unwrap().to_str().unwrap(),
        );

        if self.show_size {
            result.push_str(&self.size.unwrap().to_string());
        }

        write!(f, "{}\n", result)
    }
}

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!(
            "{}{}{}",
            self.prefix,
            self.connector,
            self.path.file_name().unwrap().to_str().unwrap(),
        );

        if self.show_size {
            result.push_str(&self.size.unwrap().to_string());
        }

        write!(f, "{}\n", result)?;
        for child in self.children_file.clone().iter() {
            write!(f, "{}", child)?;
        }
        for child in self.children_dir.clone().iter() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

pub fn build_tree(path: &PathBuf, th_depth: i32, show_size: bool) {
    //let root_path: &PathBuf;
    // for some reason powershell does not expand this
    let root_path = match path.to_str().unwrap() {
        "." => current_dir().expect("Could not identify current path"),
        _ => path.to_owned(),
    };

    let mut root_dir = DirEntry::build(
        root_path.to_owned(),
        String::from(""),
        String::from(""),
        0,
        th_depth,
        false,
    );
    root_dir.get_children();
    println!("{}", root_dir)
}
