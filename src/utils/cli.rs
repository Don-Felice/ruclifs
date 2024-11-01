use regex::Regex;
use std::io;
use std::process;

pub const INDENT: &str = "    ";

pub fn print_line(text: &str) {
    println!("{:―^50}", text);
}

pub struct Styler {
    style_seq: String,
    reset_seq: String,
    regex: Option<Regex>,
}
impl Styler {
    pub fn build(
        color_fg: &str,
        color_bg: &str,
        bold: bool,
        underline: bool,
        pattern: &str,
    ) -> Result<Styler, &'static str> {
        // do nothing if no options are chosen
        if (color_fg == "" || color_fg == "default")
            && (color_bg == "" || color_bg == "default")
            && !bold
            && !underline
        {
            return Ok(Styler {
                style_seq: String::from(""),
                reset_seq: String::from(""),
                regex: None,
            });
        }

        let mut style: Vec<&str> = Vec::new();

        if bold {
            style.push("1");
        }
        if underline {
            style.push("4");
        }

        match color_fg {
            "cyan" => style.push("96"),
            "green" => style.push("32"),
            "gray" => style.push("90"),
            "red" => style.push("31"),
            "yellow" => style.push("33"),
            "" | "default" => (),
            _ => return Err("Chosen color is not supported."),
        };

        match color_bg {
            "cyan" => style.push("46"),
            "gray" => style.push("100"),
            "red" => style.push("41"),
            "yellow" => style.push("43"),
            "" | "default" => (),
            _ => return Err("Chosen color is not supported."),
        };
        let style_str = style.join(";");

        let style_seq = format!("\x1b[{}m", style_str);

        // get regex
        if pattern != "" {
            let style_regex = Regex::new(&format!("({})", pattern)).unwrap_or_else(|err| {
                println!("Problem when compiling the regex pattern: {err}");
                process::exit(1)
            });
            return Ok(Styler {
                style_seq: style_seq.to_owned(),
                reset_seq: String::from("\x1b[0m"),
                regex: Some(style_regex),
            });
        } else {
            return Ok(Styler {
                style_seq: style_seq.to_owned(),
                reset_seq: String::from("\x1b[0m"),
                regex: None,
            });
        }
    }

    pub fn style(&self, text: &str) -> String {
        match &self.regex {
            None => format!("{}{}{}", &self.style_seq, text, &self.reset_seq),
            Some(re) => re
                .replace_all(text, format!("{}$1{}", &self.style_seq, &self.reset_seq))
                .into_owned(),
        }
    }
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
        Err(err) => println!("Problem reading the input: {err}"),
    }
}
