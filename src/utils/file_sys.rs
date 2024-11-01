use glob::glob;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process;

struct MockPaths {
    pub taken: Vec<PathBuf>,
    pub free: Vec<PathBuf>,
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
            Err(e) => {
                println!("Problem with file collection: {:?}.\nAborting.", e);
                process::exit(1)
            }
        }
    }
    files.sort();
    files
}

pub struct UniquePathGetter {
    mock_paths: MockPaths, // mimic taken and free paths when running dry
    num_regex: Regex,      // we only compile at construct time
}
impl UniquePathGetter {
    /// Initializes a UniquePathGetter with empty mocking no taken or free paths.
    pub fn new() -> UniquePathGetter {
        return UniquePathGetter {
            mock_paths: MockPaths {
                taken: Vec::new(),
                free: Vec::new(),
            },
            num_regex: Regex::new(r"_(\d*)$").unwrap(),
        };
    }

    pub fn add_mock_taken(&mut self, path: PathBuf) {
        self.mock_paths.taken.push(path)
    }

    pub fn add_mock_free(&mut self, path: PathBuf) {
        self.mock_paths.free.push(path)
    }
    pub fn get_unique(&self, path_in: &PathBuf) -> PathBuf {
        let file_stem_in = path_in.file_stem().unwrap().to_str().unwrap();
        if (path_in.exists() || self.mock_paths.taken.contains(&path_in))
            && !self.mock_paths.free.contains(&path_in)
        {
            let mut name_count: i32;

            let mut file_ext;
            match path_in.extension() {
                Some(ext) => {
                    file_ext = String::from(".");
                    file_ext.push_str(ext.to_str().unwrap())
                }
                None => file_ext = String::from(""),
            }

            let file_stem_bare: String;

            match self.num_regex.captures(file_stem_in) {
                Some(caps) => {
                    let cap = caps.get(1).unwrap().as_str();
                    name_count = cap.parse::<i32>().unwrap() + 1;
                    file_stem_bare = self.num_regex.replace_all(file_stem_in, "").to_string();
                }
                None => {
                    file_stem_bare = file_stem_in.to_owned();
                    name_count = 1;
                }
            }
            let mut file_name_new =
                file_stem_bare.clone() + &format!("_{}{}", name_count, file_ext);

            let mut path_out = path_in
                .parent()
                .expect("No parent folder identified.")
                .join(&file_name_new);

            while (path_out.exists() || self.mock_paths.taken.contains(&path_out))
                && !self.mock_paths.free.contains(&path_out)
            {
                name_count += 1;
                file_name_new = file_stem_bare.clone() + &format!("_{}{}", name_count, file_ext);
                path_out = path_in.parent().unwrap().join(&file_name_new);
            }
            return path_out;
        } else {
            return path_in.to_path_buf();
        }
    }
}

#[cfg(test)]
mod test_get_unique_path {
    use std::env::current_exe;
    use std::path::PathBuf;

    use super::UniquePathGetter;

    #[test]
    fn path_free() {
        let path_getter = UniquePathGetter::new();
        let path_in = PathBuf::from("/some/path/to/file.rs");
        let path_out = path_getter.get_unique(&path_in);
        assert_eq!(path_out, path_in);
    }

    #[test]
    fn path_taken() {
        let path_getter = UniquePathGetter::new();
        let path_in = current_exe().unwrap();
        let path_out = path_getter.get_unique(&path_in);
        assert_eq!(path_out.parent(), path_in.parent());
        assert_ne!(path_out, path_in);
    }

    #[test]
    fn path_in_mocked_taken() {
        let mut path_getter = UniquePathGetter::new();
        let path_in = PathBuf::from("/some/path/to/file.rs");
        path_getter.add_mock_taken(path_in.clone());
        let path_out = path_getter.get_unique(&path_in);
        assert_eq!(path_out, PathBuf::from("/some/path/to/file_1.rs"));
    }

    #[test]
    fn path_in_mocked_taken_has_suffix() {
        let mut path_getter = UniquePathGetter::new();
        let path_in = PathBuf::from("/some/path/to/file_41.rs");
        path_getter.add_mock_taken(path_in.clone());
        let path_out = path_getter.get_unique(&path_in);
        assert_eq!(path_out, PathBuf::from("/some/path/to/file_42.rs"));
    }

    #[test]
    fn path_in_mocked_free() {
        let mut path_getter = UniquePathGetter::new();
        let path_in = current_exe().unwrap();
        path_getter.add_mock_free(path_in.clone());
        let path_out = path_getter.get_unique(&path_in);
        assert_eq!(path_out, path_in)
    }
}
