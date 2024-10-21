use glob::glob;
use regex::Regex;
use std::path::{Path, PathBuf};

pub struct MockPaths {
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
            Err(e) => println!("{:?}", e),
        }
    }
    files.sort();
    files
}

pub fn get_unique_path(path_in: &PathBuf, mock_paths: &MockPaths) -> PathBuf {
    let file_name = path_in.file_name().unwrap().to_str().unwrap();
    if (path_in.exists() || mock_paths.taken.contains(&path_in))
        && !mock_paths.free.contains(&path_in)
    {
        let mut name_count: i32;
        let file_ext = path_in.extension().unwrap().to_str().unwrap();
        let re = Regex::new(&format!(r"_(\d*).{}$", file_ext)).unwrap();
        let mut file_name_new: String;

        match re.captures(file_name) {
            Some(caps) => {
                let cap = caps.get(1).unwrap().as_str();
                name_count = cap.parse::<i32>().unwrap() + 1;
                file_name_new = re
                    .replace_all(file_name, format!("_{}.{}", name_count, file_ext))
                    .to_string();
            }
            None => {
                file_name_new = path_in.file_stem().unwrap().to_str().unwrap().to_owned();
                file_name_new.push_str("_1.");
                file_name_new.push_str(file_ext);
                name_count = 1;
            }
        }

        let mut path_out = path_in
            .parent()
            .expect("No parent folder identified.")
            .join(&file_name_new);

        while (path_out.exists() || mock_paths.taken.contains(&path_out))
            && !mock_paths.free.contains(&path_out)
        {
            name_count += 1;
            file_name_new = re
                .replace_all(&file_name_new, format!("_{}.{}", name_count, file_ext))
                .to_string();
            path_out = path_in.parent().unwrap().join(&file_name_new);
        }
        return path_out;
    } else {
        return path_in.to_path_buf();
    }
}

#[cfg(test)]
mod test_get_unique_path {
    use std::env::current_exe;
    use std::path::PathBuf;

    use super::{get_unique_path, MockPaths};

    #[test]
    fn path_free() {
        let path_in = PathBuf::from("/some/path/to/file.rs");
        let path_out = get_unique_path(
            &path_in,
            &MockPaths {
                taken: Vec::new(),
                free: Vec::new(),
            },
        );
        assert_eq!(path_out, path_in);
    }

    #[test]
    fn path_taken() {
        let path_in = current_exe().unwrap();
        let path_out = get_unique_path(
            &path_in,
            &MockPaths {
                taken: Vec::new(),
                free: Vec::new(),
            },
        );
        assert_eq!(path_out.parent(), path_in.parent());
        assert_ne!(path_out, path_in);
    }

    #[test]
    fn path_in_mocked_taken() {
        let path_in = PathBuf::from("/some/path/to/file.rs");
        let path_out = get_unique_path(
            &path_in,
            &MockPaths {
                taken: vec![path_in.clone()],
                free: Vec::new(),
            },
        );
        assert_eq!(path_out, PathBuf::from("/some/path/to/file_1.rs"));
    }

    #[test]
    fn path_in_mocked_free() {
        let path_in = current_exe().unwrap();
        let path_out = get_unique_path(
            &path_in,
            &MockPaths {
                taken: Vec::new(),
                free: vec![path_in.clone()],
            },
        );
        assert_eq!(path_out, path_in)
    }
}
