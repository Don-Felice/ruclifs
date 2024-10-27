use crate::utils::cli::{print_line, proceed_query, Styler, INDENT};
use crate::utils::file_sys::{get_files, get_unique_path, MockPaths};
use clap::builder::ArgAction;
use clap::Args;
use regex::Regex;
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
    pattern: &str,
    substitute: &str,
    dry_run: bool,
    mock_paths: &MockPaths,
) -> PathBuf {
    let file_name = path_file.file_name().unwrap().to_str().unwrap();
    let re = Regex::new(pattern).unwrap_or_else(|err| {
        println!("Problem when compiling the regex pattern: {err}");
        process::exit(1)
    });
    let file_name_new = re.replace_all(file_name, substitute).to_string();
    let styler_warning = Styler::build("yellow", "", false, false, "").unwrap();
    let styler_grayed = Styler::build("gray", "", false, false, "").unwrap();
    let styler_match = Styler::build("cyan", "", false, true, pattern).unwrap();

    if file_name_new != file_name {
        let file_name_color = styler_match.style(file_name);

        let path_candidate = path_file.parent().unwrap().join(file_name_new);
        let path_new = get_unique_path(&path_candidate, mock_paths);

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
        return path_new;
    } else {
        let printout = styler_grayed.style(format!("{} -> {}", file_name, file_name).as_str());

        println!("{printout}");
        return path_file.to_path_buf();
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
        let mut mock_paths = MockPaths {
            taken: Vec::new(),
            free: Vec::new(),
        };
        print_line("PREVIEW");
        for file in &files {
            let path_new = rename_file(file, pattern, substitute, true, &mock_paths);
            // mock new file structure after renaming
            if &path_new != file {
                mock_paths.taken.push(path_new);
                mock_paths.free.push(file.to_path_buf());
            }
        }
        print_line("END PREVIEW");
        proceed_query("If you wanna apply this renaming, give me a 'yes' or 'y' now:");
    }
    print_line("");
    let empty_mock_paths = MockPaths {
        taken: Vec::new(),
        free: Vec::new(),
    };
    for file in &files {
        let _ = rename_file(file, pattern, substitute, false, &empty_mock_paths);
    }
    print_line("");
}
