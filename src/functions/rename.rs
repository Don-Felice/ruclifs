use crate::utils::cli::{print_line, proceed_query, Styler, INDENT};
use crate::utils::file_sys::{get_files, UniquePathGetter};
use clap::builder::ArgAction;
use clap::Args;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;
use std::process;

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

fn rename_file(
    path_file: &PathBuf,
    regex: &Regex,
    substitute: &str,
    dry_run: bool,
    match_styler: &Styler,
    path_getter: &UniquePathGetter,
) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = path_file.file_name().unwrap().to_str().unwrap();
    let file_name_new = regex.replace_all(file_name, substitute).to_string();
    let styler_warning = Styler::build("yellow", "", false, false, "").unwrap();
    let styler_grayed = Styler::build("gray", "", false, false, "").unwrap();

    if file_name_new != file_name {
        let file_name_color = match_styler.style(file_name);

        let path_candidate = path_file.parent().unwrap().join(file_name_new);
        let path_new = path_getter.get_unique(&path_candidate);

        let mut print_message = format!(
            "{} -> {}",
            file_name_color,
            path_new.file_name().unwrap().to_str().unwrap()
        );

        if path_new != path_candidate {
            print_message.push_str(INDENT);
            print_message
                .push_str(&styler_warning.style("Warning: Path already exists, adding suffix."))
        }

        println!("{}", print_message);
        if !dry_run {
            let _ = std::fs::rename(path_file, path_new.clone());
        }
        return Ok(path_new);
    } else {
        let printout = styler_grayed.style(format!("{} -> {}", file_name, file_name).as_str());

        println!("{printout}");
        return Ok(path_file.to_path_buf());
    }
}

pub fn rename(
    path: &std::path::PathBuf,
    filter_string: &str,
    pattern: &str,
    substitute: &str,
    recursive: bool,
    skip_preview: bool,
) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(pattern).unwrap_or_else(|err| {
        println!("Problem when compiling the regex pattern: {err}");
        process::exit(1)
    });
    let match_styler = Styler::build("cyan", "", false, true, pattern).unwrap();

    // get file to rename
    let files = get_files(path, filter_string, recursive);
    println!("Renaming {} files:", files.len());

    if !skip_preview {
        let mut path_getter = UniquePathGetter::new();
        print_line("PREVIEW");
        for file in &files {
            let path_new =
                rename_file(file, &regex, substitute, true, &match_styler, &path_getter)?;
            // mock new file structure after renaming
            if &path_new != file {
                path_getter.add_mock_taken(path_new);
                path_getter.add_mock_free(file.to_path_buf());
            }
        }
        print_line("END PREVIEW");
        proceed_query("If you wanna rename for real, give me a 'yes' or 'y' now:");
    }
    print_line("");
    let path_getter = UniquePathGetter::new();
    for file in &files {
        let _ = rename_file(file, &regex, substitute, false, &match_styler, &path_getter)?;
    }
    print_line("");
    Ok(())
}

#[cfg(test)]
mod test_rename {
    use std::fs::{create_dir, File};
    use tempfile::tempdir;

    use super::rename;

    #[test]
    fn rename_files_recursive() {
        let tempdir = tempdir().unwrap();
        let tempdir_path = tempdir.path().to_path_buf();

        let file_path = tempdir_path.join("some_file.txt");
        File::create(file_path).unwrap();
        let file_path = tempdir_path.join("some_other_file.txt");
        File::create(file_path).unwrap();

        let subdir = tempdir.path().join("subdir");
        create_dir(&subdir).unwrap();
        File::create(subdir.join("some_file.txt")).unwrap();

        rename(&tempdir_path, "*", "some", "other", true, true).unwrap();

        assert!(!tempdir_path.join("some_file.txt").is_file());
        assert!(tempdir_path.join("other_file.txt").is_file());

        assert!(!tempdir_path.join("some_other_file.txt").is_file());
        assert!(tempdir_path.join("other_other_file.txt").is_file());

        assert!(!subdir.join("some_file.txt").is_file());
        assert!(subdir.join("other_file.txt").is_file());

        tempdir.close().unwrap();
    }

    #[test]
    fn rename_files_non_recursive() {
        let tempdir = tempdir().unwrap();
        let tempdir_path = tempdir.path().to_path_buf();

        let file_path = tempdir_path.join("some_file.txt");
        File::create(file_path).unwrap();

        let subdir = tempdir_path.join("subdir.txt");
        create_dir(&subdir).unwrap();
        File::create(subdir.join("some_file.txt")).unwrap();

        rename(&tempdir_path, "*", "some", "other", false, true).unwrap();

        assert!(!tempdir_path.join("some_file.txt").is_file());
        assert!(tempdir_path.join("other_file.txt").is_file());

        assert!(!subdir.join("other_file.txt").is_file());
        assert!(subdir.join("some_file.txt").is_file());

        tempdir.close().unwrap();
    }

    #[test]
    fn rename_files_filtered() {
        let tempdir = tempdir().unwrap();
        let tempdir_path = tempdir.path().to_path_buf();

        let file_path = tempdir.path().join("some_file.txt");
        File::create(file_path).unwrap();
        let file_path = tempdir.path().join("some_other_file.txt");
        File::create(file_path).unwrap();

        rename(&tempdir_path, "*other*", "some", "other", true, true).unwrap();

        assert!(tempdir_path.join("some_file.txt").is_file());
        assert!(!tempdir_path.join("other_file.txt").is_file());

        assert!(!tempdir_path.join("some_other_file.txt").is_file());
        assert!(tempdir_path.join("other_other_file.txt").is_file());

        tempdir.close().unwrap();
    }
}
