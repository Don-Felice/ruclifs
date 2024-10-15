use regex::Regex;
use std::path::PathBuf;

pub struct MockPaths {
    pub taken: Vec<PathBuf>,
    pub free: Vec<PathBuf>,
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
