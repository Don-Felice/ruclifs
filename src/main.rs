mod functions;

use clap::{Args, Parser, Subcommand};

use crate::functions::rename::{rename, RenameArgs};
use crate::functions::sed::SedArgs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct MainArgs {
    #[command(subcommand)]
    cmd: Commands, // path_file: std::path::PathBuf,

                   // #[arg(short = 'p', long = "pattern")]
                   // pattern: String,

                   // #[arg(short = 'r', long = "replacement", default_value_t = String::from(" "))]
                   // replacement: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Renaming files and so on
    Rename(RenameArgs),
    ///Awesome Streaming editor
    Sed(SedArgs),
    // copy(CopyArgs),
    // delete(DeleteArgs),
}

fn color_string(s: &str) -> String {
    format!("\x1b[34;4m{s}\x1b[0m")
}

fn main() {
    let version: &str = "0.0.0";
    println!("\x1b[90mThis is ruclifs version {version}\x1b[0m");

    let args = MainArgs::parse();

    match &args.cmd {
        Commands::Rename(cmd_args) => {
            println!("{:?}", cmd_args);
            rename(
                &cmd_args.path,
                &cmd_args.filter_string,
                &cmd_args.pattern,
                &cmd_args.substitute,
                cmd_args.recursive,
                cmd_args.skip_preview,
            );
        }
        Commands::Sed(cmd_args) => {
            println!("{:?}", cmd_args)
        }
        _ => {}
    }

    // println!(
    //     "path_file: {:?}, pattern: {:?}, replacement: {:?}",
    //     args.path_file, args.pattern, args.replacement
    // );

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
    // }
}
