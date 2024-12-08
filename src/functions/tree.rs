use crate::utils::cli::{bites2str, Styler};
use clap::builder::ArgAction;
use clap::Args;
use std::env::current_dir;
use std::fmt;
use std::fs::{self, metadata};
use std::path::PathBuf;

const ELBOW: &str = "└── ";
const TEE: &str = "├── ";
const PIPE_PREFIX: &str = "│   ";
const SPACE_PREFIX: &str = "    ";

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
struct FileEntry<'a> {
    path: PathBuf,
    prefix: String,
    connector: String,
    show_size: bool,
    size: Option<u64>,
    styler_size: &'a Styler,
}

#[derive(Clone)]
struct DirEntry<'a> {
    path: PathBuf,
    prefix: String,
    connector: String,
    depth: i32,
    th_depth: i32,
    show_size: bool,
    have_access: bool,
    children_file: Vec<FileEntry<'a>>,
    children_dir: Vec<DirEntry<'a>>,
    size: Option<u64>,
    styler_size: &'a Styler,
    styler_folder: &'a Styler,
}

impl FileEntry<'_> {
    fn build(
        path: PathBuf,
        prefix: String,
        connector: String,
        show_size: bool,
        styler_size: &Styler,
    ) -> FileEntry {
        let size: Option<u64>;
        if show_size {
            size = Some(metadata(&path).unwrap().len());
        } else {
            size = None;
        }
        FileEntry {
            path: path,
            prefix: prefix,
            connector: connector,
            show_size: show_size,
            size: size,
            styler_size: styler_size,
        }
    }
}

impl DirEntry<'_> {
    fn build<'a>(
        path: PathBuf,
        prefix: String,
        connector: String,
        depth: i32,
        th_depth: i32,
        show_size: bool,
        styler_size: &'a Styler,
        styler_folder: &'a Styler,
    ) -> DirEntry<'a> {
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
            size: None,
            styler_size: styler_size,
            styler_folder: styler_folder,
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

        let mut content: Vec<PathBuf>;

        match fs::read_dir(self.path.clone()) {
            Ok(c) => {
                content = c
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect::<Vec<_>>();
                content.sort_by_key(|e| (e.is_dir(), e.to_owned()))
            }
            Err(_) => {
                self.have_access = false;
                content = Vec::new()
            }
        };

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
                    self.styler_size,
                    self.styler_folder,
                );
                new_dir_entry.get_children();
                children_dir.push(new_dir_entry);
            } else {
                children_file.push(FileEntry::build(
                    path.to_path_buf(),
                    child_prefix.clone(),
                    child_connector,
                    self.show_size,
                    self.styler_size,
                ));
            }
        }
        self.children_file = children_file;
        self.children_dir = children_dir;
    }

    fn get_size(&mut self) {
        if !self.have_access {
            self.size = None;
            return;
        }
        let mut all_access = true;
        let mut size: u64 = 0;
        for i in self.children_dir.iter_mut() {
            i.get_size();
            if i.size == None {
                all_access = false;
            }
            size += i.size.unwrap_or_default();
        }
        for i in self.children_file.iter() {
            size += i.size.unwrap_or_default();
        }
        self.size = if all_access { Some(size) } else { None };
    }
}

impl fmt::Display for FileEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!(
            "{}{}{}",
            self.prefix,
            self.connector,
            self.path.file_name().unwrap().to_str().unwrap(),
        );

        if self.show_size {
            result.push_str(
                format!(" {:6}", bites2str(self.size.unwrap(), self.styler_size,)).as_str(),
            );
        }

        write!(f, "{}\n", result)
    }
}

impl fmt::Display for DirEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!(
            "{}{}{}",
            self.prefix,
            self.connector,
            self.styler_folder
                .style(self.path.file_name().unwrap().to_str().unwrap()),
        );

        if self.show_size {
            let size_suffix: String;

            if self.have_access {
                size_suffix = match self.size {
                    Some(c) => format!(" {:6}", bites2str(c, self.styler_size)),
                    None => self.styler_size.style(" size unknown"),
                }
            } else {
                size_suffix = String::from("  \u{1b}[31maccess error\u{1b}[0m")
            };
            result.push_str(size_suffix.as_str());
        }
        write!(f, "{}\n", result)?;

        // print children if depth level permits
        if (self.th_depth <= 0) || (self.depth < self.th_depth) {
            for child in self.children_file.clone().iter() {
                write!(f, "{}", child)?;
            }
            for child in self.children_dir.clone().iter() {
                write!(f, "{}", child)?;
            }
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

    let styler_size = Styler::build("cyan", "", false, false, "").unwrap();
    let styler_folder = Styler::build("yellow", "", false, false, "").unwrap();

    let mut root_dir = DirEntry::build(
        root_path.to_owned(),
        String::from(""),
        String::from(""),
        0,
        th_depth,
        show_size,
        &styler_size,
        &styler_folder,
    );
    root_dir.get_children();
    if show_size {
        root_dir.get_size();
    }
    println!("{}", root_dir)
}

#[cfg(test)]
mod test_tree {
    use crate::utils::cli::Styler;
    use std::fs::{create_dir, File};
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::DirEntry;

    fn set_up_dir(dir: &Path) -> PathBuf {
        let rootdir = dir.join("root_dir");
        create_dir(&rootdir).unwrap();

        File::create(rootdir.join("some_file_1.txt")).unwrap();
        File::create(rootdir.join("some_file_2.txt")).unwrap();

        let some_subdir = rootdir.join("some_subdir");
        create_dir(&some_subdir).unwrap();
        let some_other_subdir = rootdir.join("some_other_subdir");
        create_dir(&some_other_subdir).unwrap();

        File::create(some_subdir.join("some_subdir_file_1.txt")).unwrap();
        File::create(some_subdir.join("some_subdir_file_2.rs")).unwrap();

        let some_subsubdir = rootdir.join("some_subsubdir");
        create_dir(&some_subsubdir).unwrap();

        File::create(some_subsubdir.join("some_subsubdir_file_1.txt")).unwrap();
        File::create(some_subsubdir.join("some_subsubdir_file_2.rs")).unwrap();
        File::create(some_subsubdir.join("some_subsubdir_file_3.rs")).unwrap();

        return rootdir;
    }

    #[test]
    fn tree_full_depth_no_size() {
        // set up directory
        let tempdir = tempdir().unwrap();
        let rootdir = set_up_dir(&tempdir.path());

        let styler_size = Styler::build("", "", false, false, "").unwrap();
        let styler_folder = Styler::build("", "", false, false, "").unwrap();
        // build tree
        let mut root_dir_entry = DirEntry::build(
            rootdir.to_owned(),
            String::from(""),
            String::from(""),
            0,
            -1,
            false,
            &styler_size,
            &styler_folder,
        );
        root_dir_entry.get_children();

        println!("{}", root_dir_entry);

        assert_eq!(
            root_dir_entry.to_string(),
            "\
root_dir
├── some_file_1.txt
├── some_file_2.txt
├── some_other_subdir
├── some_subdir
│   ├── some_subdir_file_1.txt
│   └── some_subdir_file_2.rs
└── some_subsubdir
    ├── some_subsubdir_file_1.txt
    ├── some_subsubdir_file_2.rs
    └── some_subsubdir_file_3.rs
"
        );

        // teardown
        tempdir.close().unwrap();
    }

    #[test]
    fn tree_full_depth() {
        // set up directory
        let tempdir = tempdir().unwrap();
        let rootdir = set_up_dir(&tempdir.path());

        let styler_size = Styler::build("", "", false, false, "").unwrap();
        let styler_folder = Styler::build("", "", false, false, "").unwrap();
        // build tree
        let mut root_dir_entry = DirEntry::build(
            rootdir.to_owned(),
            String::from(""),
            String::from(""),
            0,
            -1,
            true,
            &styler_size,
            &styler_folder,
        );
        root_dir_entry.get_children();
        root_dir_entry.get_size();

        println!("{}", root_dir_entry);

        assert_eq!(
            root_dir_entry.to_string(),
            "\
root_dir    0.00 B
├── some_file_1.txt    0.00 B
├── some_file_2.txt    0.00 B
├── some_other_subdir    0.00 B
├── some_subdir    0.00 B
│   ├── some_subdir_file_1.txt    0.00 B
│   └── some_subdir_file_2.rs    0.00 B
└── some_subsubdir    0.00 B
    ├── some_subsubdir_file_1.txt    0.00 B
    ├── some_subsubdir_file_2.rs    0.00 B
    └── some_subsubdir_file_3.rs    0.00 B
"
        );

        // teardown
        tempdir.close().unwrap();
    }
}
