use crate::utils::cli::Styler;
use crate::utils::file_sys::read_lines;
use anyhow::{anyhow, Result};
use clap::builder::ArgAction;
use clap::Args;
use regex::Regex;
use std::fs::{rename, File};
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::process;

#[derive(Args, Debug)]
pub struct SedArgs {
    pub path_file: std::path::PathBuf,
    #[arg(short = 'p', long = "pattern")]
    pub pattern: String,
    #[arg(short = 's', long = "substitute")]
    pub substitute: String,
    #[arg(short = 'r', long = "recursive",action=ArgAction::SetTrue)]
    pub recursive: bool,
    #[arg(short = 'o', long = "overwrite",action=ArgAction::SetTrue)]
    pub overwrite: bool,
}

struct StreamingEditor {
    regex: Regex,
    substitute: String,
    styler_match: Styler,
}

impl StreamingEditor {
    fn build(pattern: String, substitute: String) -> Result<StreamingEditor> {
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
        let styler = Styler::build("cyan", "", false, true, &pattern)?;

        return Ok(StreamingEditor {
            regex: regex,
            substitute: substitute,
            styler_match: styler,
        });
    }

    fn edit(
        &self,
        path_file: &PathBuf,
        lines: String,
        max_previews: i32,
        encoding: String,
        overwrite: bool,
        preview_mode: bool,
    ) {
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
                    let mut line_new = self.regex.replace_all(&line, &self.substitute).to_string();
                    line_new.push_str("\n");
                    println!("l{} old: {}", n, &self.styler_match.style(&line));
                    println!("l{} new: {}", n, &line_new);
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
}

pub fn edit_files(
    path_file: &PathBuf,
    pattern: &String,
    substitute: &String,
    overwrite: bool,
    recursive: bool,
) {
    let editor = match StreamingEditor::build(pattern.clone(), substitute.clone()) {
        Ok(r) => r,
        Err(err) => {
            println!("Looks like there was an issue:\n{err}");
            process::exit(1)
        }
    };
    editor.edit(
        path_file,
        String::from(""),
        3,
        String::from("UTF-8"),
        overwrite,
        true,
    );
}

// let re = Regex::new(&args.pattern).unwrap();
// let highlight_pattern = format!("({})", &args.pattern);
// let hightlight_re = Regex::new(&highlight_pattern).unwrap();
// println!("highlight pattern: {:?}", highlight_pattern);

// let content: String = std::fs::read_to_string(&args.path_file).expect("could not read file");

// for line in content.lines() {
//     let result_line = re.replace_all(line, &args.replacement);
//     let highlight_line = hightlight_re.replace_all(line, color_string("$1"));
//     println!("{}", highlight_line);
//     println!("-> {}", result_line);
