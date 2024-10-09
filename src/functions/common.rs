use ::phf::{phf_map, Map};
use regex::Regex;
use std::io;

pub const COLORS: Map<&str, &str> = phf_map! {
    "grey" => "\x1b[90m",
    "red"=> "\x1b[32m",
    "cyan"=> "\x1b[96m",
    "default"=> "\x1b[0m",
};

pub fn highlight_string(s: &str, color: &str, underline: bool) -> String {
    let mut highlight = color.to_owned();
    if underline {
        highlight.pop().unwrap();
        highlight.push_str(";4m");
    }
    format!("{}{}{}", highlight, s, COLORS.get("default").unwrap())
}

pub fn color_substring(string: &str, pattern: &str, color: &str, underline: bool) -> String {
    let highlight_pattern = format!("({})", pattern);
    let hightlight_re = Regex::new(&highlight_pattern).unwrap();
    hightlight_re
        .replace_all(string, highlight_string("$1", color, underline))
        .into_owned()
}

pub fn proceed_query(text: &str) {
    println!("\n{}", text);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if input.trim() != "yes" && input.trim() != "y" {
                println!("Will abort here. See you soon!");
                std::process::exit(0)
            }
        }
        Err(error) => println!("error: {error}"),
    }
}
