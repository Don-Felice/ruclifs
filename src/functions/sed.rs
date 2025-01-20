use crate::utils::cli::{print_line, proceed_query, AnsiColor, Styler};
use crate::utils::file_sys::{read_lines, GetFilesArgs};
use anyhow::{anyhow, Result};
use clap::builder::ArgAction;
use clap::Args;
use regex::Regex;
use std::fs::{rename, File};
use std::io::{LineWriter, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::process;

#[derive(Args, Debug)]
pub struct SedArgs {
    #[command(flatten)]
    pub get_files_args: GetFilesArgs,
    #[arg(short = 'p', long = "pattern")]
    pub pattern: String,
    #[arg(short = 's', long = "substitute")]
    pub substitute: String,
    #[arg(short = 'l', long = "lines", default_value = "")]
    pub lines: String,
    #[arg(short = 'v', long = "preview_max", default_value_t = 3)]
    pub preview_max: i32,
    #[arg(short = 'o', long = "overwrite",action=ArgAction::SetTrue)]
    pub overwrite: bool,
}

struct StreamingEditor {
    regex: Regex,
    substitute: String,
    styler_match: Styler,
    lines: LineSelection,
}

pub enum LineSelection {
    LRange(Range<usize>),
    Numbers(Vec<usize>),
    All,
}

impl LineSelection {
    fn contains(&self, num: &usize) -> bool {
        match self {
            LineSelection::LRange(r) => return r.contains(num),
            LineSelection::Numbers(v) => return v.contains(num),
            LineSelection::All => return true,
        }
    }
}

impl StreamingEditor {
    fn build(pattern: String, substitute: String, lines: String) -> Result<StreamingEditor> {
        let regex = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(err) => {
                return Err(anyhow!(
                    "Error when trying to compile a regex from '{:?}':\n{}",
                    pattern,
                    err
                ));
            }
        };
        let styler = Styler::build(&AnsiColor::Cyan, &AnsiColor::Default, false, true, &pattern)?;

        return Ok(StreamingEditor {
            regex: regex,
            substitute: substitute,
            styler_match: styler,
            lines: StreamingEditor::parse_lines(lines)?,
        });
    }

    fn edit_preview(
        &self,
        path_file: &PathBuf,
        preview_max: &i32,
        mut match_count: i32,
    ) -> Result<i32> {
        let styler_dimmed =
            Styler::build(&AnsiColor::Gray, &AnsiColor::Default, false, false, "").unwrap();
        println!("{}", styler_dimmed.style(&path_file.to_str().unwrap()));
        let match_count_file = match_count.clone();
        match read_lines(path_file) {
            // Consumes the iterator, returns an (Optional) String
            Ok(lines) => {
                for (n, line) in lines.flatten().enumerate() {
                    let l = n + 1;
                    let line_new = if self.lines.contains(&l) {
                        self.regex.replace_all(&line, &self.substitute).to_string()
                    } else {
                        line.clone()
                    };

                    if line != line_new {
                        match_count += 1;
                        println!("  l{} old: {}", l, &self.styler_match.style(&line));
                        println!("  l{} new: {}", l, &line_new);
                        println!("");
                    }

                    if &match_count >= preview_max {
                        break;
                    }
                }
                if match_count == match_count_file {
                    println!("{}\n", styler_dimmed.style("  --"));
                }
                return Ok(match_count);
            }
            Err(e) => {
                return Err(anyhow!(
                    "Failed to read lines from file: \n {}. Error:\n{}",
                    path_file.to_str().unwrap(),
                    e
                ))
            }
        }
    }

    fn edit(&self, path_file: &PathBuf, overwrite: bool) {
        // TODO: currently there is also a newline added in the modified file if the last line of the file ends with EOF
        match read_lines(path_file) {
            // Consumes the iterator, returns an (Optional) String
            Ok(lines) => {
                let filename_out = format!(
                    "{}_edited.{}",
                    path_file.file_stem().unwrap().to_str().unwrap(),
                    path_file.extension().unwrap().to_str().unwrap()
                );
                let path_out_file = path_file.parent().unwrap().join(&filename_out);

                let file = File::create(&path_out_file).unwrap();

                let mut file = LineWriter::new(file);

                for (n, line) in lines.flatten().enumerate() {
                    let l = n + 1;

                    let mut line_new = if self.lines.contains(&l) {
                        self.regex.replace_all(&line, &self.substitute).to_string()
                    } else {
                        line.clone()
                    };
                    line_new.push_str("\n");

                    file.write_all(line_new.as_bytes()).unwrap();
                }
                file.flush().unwrap();
                if overwrite {
                    rename(path_out_file, path_file).unwrap();
                }
            }
            Err(e) => {
                println!("Error when accessing the file: {e}");
                process::exit(1);
            }
        }
    }

    fn parse_lines(input: String) -> Result<LineSelection> {
        let regex_range = Regex::new(r"^\d+-\d+$").unwrap();
        let regex_vec = Regex::new(r"^\d+(,\d+)*$").unwrap();

        if input == "" {
            return Ok(LineSelection::All);
        } else if regex_range.is_match(&input) {
            let min_max: Vec<usize> = input
                .split("-")
                .map(|x| x.parse::<usize>().unwrap())
                .collect();
            return Ok(LineSelection::LRange(Range {
                start: min_max[0],
                end: min_max[1] + 1,
            }));
        } else if regex_vec.is_match(&input) {
            return Ok(LineSelection::Numbers(
                input
                    .split(",")
                    .map(|x| x.parse::<usize>().unwrap())
                    .collect(),
            ));
        } else {
            return Err(anyhow!(
                "Failed to parse line input '{}'. \
                Please either provide a list of line numbers separated by ',', or a range given as 'start-end'",
                input
            ));
        }
    }
}

pub fn edit_files(
    file_paths: &Vec<PathBuf>,
    pattern: &String,
    substitute: &String,
    lines: &String,
    preview_max: &i32,
    overwrite: bool,
) {
    let styler_error =
        Styler::build(&AnsiColor::Red, &AnsiColor::Default, false, false, "").unwrap();
    let editor = match StreamingEditor::build(pattern.clone(), substitute.clone(), lines.clone()) {
        Ok(r) => r,
        Err(err) => {
            println!("{}", styler_error.style("Looks like there was an issue:"));
            println!("{err}");
            process::exit(1)
        }
    };
    if preview_max > &0 {
        print_line("PREVIEW");
        let mut match_count = 0;
        for file in file_paths.iter() {
            match editor.edit_preview(file, preview_max, match_count) {
                Ok(num_matches) => {
                    match_count = num_matches;
                    if &match_count >= preview_max {
                        break;
                    }
                }
                Err(err) => {
                    println!("{}", styler_error.style("Looks like there was an issue:"));
                    println!("{err}");
                    process::exit(1)
                }
            };
        }
        print_line("END PREVIEW");
        proceed_query("If you wanna apply the edits, give me a 'yes' or 'y' now:");
    }
    for file in file_paths.iter() {
        editor.edit(file, overwrite);
    }
}

#[cfg(test)]
mod test_streaming_editor {
    use std::fs::{create_dir, read_to_string, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use super::StreamingEditor;

    fn set_up_files(dir: &PathBuf) -> PathBuf {
        let rootdir = dir.join("root_dir");
        create_dir(&rootdir).unwrap();
        let file_path = rootdir.join("some_file_1.txt");
        let mut file = File::create(&file_path).unwrap();

        let content = "\
l1 some example line,
l2 some next example line,
l3 some next next example line
";
        file.write_all(&content.as_bytes()).unwrap();

        return file_path;
    }
    #[test]
    fn all_lines() {
        let tempdir = tempdir().unwrap();
        let path_file = set_up_files(&tempdir.into_path());
        let content_before = read_to_string(&path_file).unwrap();

        let editor = StreamingEditor::build(
            String::from("([^ ]*ex[^ ]*)"),
            String::from("($1)"),
            String::from(""),
        )
        .unwrap();
        editor.edit(&path_file, true);

        let content_after = read_to_string(&path_file).unwrap();

        let exp_content = "\
l1 some (example) line,
l2 some (next) (example) line,
l3 some (next) (next) (example) line
";
        assert_ne!(content_before, content_after);
        assert_eq!(exp_content, content_after);
    }

    #[test]
    fn one_line() {
        let tempdir = tempdir().unwrap();
        let path_file = set_up_files(&tempdir.into_path());
        let content_before = read_to_string(&path_file).unwrap();

        let editor = StreamingEditor::build(
            String::from("([^ ]*ex[^ ]*)"),
            String::from("($1)"),
            String::from("1"),
        )
        .unwrap();
        editor.edit(&path_file, true);

        let content_after = read_to_string(&path_file).unwrap();

        let exp_content = "\
l1 some (example) line,
l2 some next example line,
l3 some next next example line
";
        assert_ne!(content_before, content_after);
        assert_eq!(exp_content, content_after);
    }

    #[test]
    fn multiple_lines() {
        let tempdir = tempdir().unwrap();
        let path_file = set_up_files(&tempdir.into_path());
        let content_before = read_to_string(&path_file).unwrap();

        let editor = StreamingEditor::build(
            String::from("([^ ]*ex[^ ]*)"),
            String::from("($1)"),
            String::from("1,3"),
        )
        .unwrap();
        editor.edit(&path_file, true);

        let content_after = read_to_string(&path_file).unwrap();

        let exp_content = "\
l1 some (example) line,
l2 some next example line,
l3 some (next) (next) (example) line
";
        assert_ne!(content_before, content_after);
        assert_eq!(exp_content, content_after);
    }

    #[test]
    fn line_range() {
        let tempdir = tempdir().unwrap();
        let path_file = set_up_files(&tempdir.into_path());
        let content_before = read_to_string(&path_file).unwrap();

        let editor = StreamingEditor::build(
            String::from("([^ ]*ex[^ ]*)"),
            String::from("($1)"),
            String::from("2-3"),
        )
        .unwrap();
        editor.edit(&path_file, true);

        let content_after = read_to_string(&path_file).unwrap();

        let exp_content = "\
l1 some example line,
l2 some (next) (example) line,
l3 some (next) (next) (example) line
";
        assert_ne!(content_before, content_after);
        assert_eq!(exp_content, content_after);
    }
}
