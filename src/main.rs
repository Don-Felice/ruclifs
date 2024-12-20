mod functions;
mod utils;
use std::process;

use clap::{Parser, Subcommand};

use crate::functions::rename::{rename, RenameArgs};
use crate::functions::sed::{edit_files, SedArgs};
use crate::functions::tree::{build_tree, TreeArgs};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Rust-based command line interface for file system operations.",
    long_about
)]
struct MainArgs {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Renaming files and so on
    Ren(RenameArgs),
    ///Awesome Streaming editor
    Sed(SedArgs),
    // copy(CopyArgs),
    // delete(DeleteArgs),
    ///Show a directory tree
    Tree(TreeArgs),
}

fn main() {
    let version: &str = "0.0.0";
    println!("\x1b[90mThis is ruclifs version {version}\x1b[0m");

    let args = MainArgs::parse();

    match &args.cmd {
        Commands::Ren(cmd_args) => {
            if let Err(e) = rename(
                &cmd_args.path,
                &cmd_args.filter_string,
                &cmd_args.pattern,
                &cmd_args.substitute,
                cmd_args.recursive,
                cmd_args.skip_preview,
            ) {
                println!("Error when renaming: {e}");
                process::exit(1);
            }
        }
        Commands::Sed(cmd_args) => {
            edit_files(
                &cmd_args.path_file.clone(),
                &cmd_args.pattern.clone(),
                &cmd_args.substitute.clone(),
                &cmd_args.lines.clone(),
                cmd_args.overwrite,
                cmd_args.recursive,
            );
        }
        Commands::Tree(cmd_args) => {
            build_tree(&cmd_args.path, cmd_args.depth, !cmd_args.hide_size);
        }
    }
}
