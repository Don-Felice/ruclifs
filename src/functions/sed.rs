use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug)]
pub struct SedArgs {
    path_file: std::path::PathBuf,
    #[arg(short = 'f', long = "filter")]
    pattern: String,
    #[arg(short = 's', long = "substitute")]
    substitute: String,
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,
}
