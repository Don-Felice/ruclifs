use anyhow::{anyhow, Result};
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
    ) -> Result<Styler> {
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
            _ => return Err(anyhow!("Chosen color is not supported:{}", color_fg)),
        };

        match color_bg {
            "cyan" => style.push("46"),
            "green" => style.push("42"),
            "gray" => style.push("100"),
            "red" => style.push("41"),
            "yellow" => style.push("43"),
            "" | "default" => (),
            //_ => return Err(format!("Chosen color is not supported:{}", "cla" ).as_str()),
            _ => return Err(anyhow!("Chosen color is not supported:{}", color_bg)),
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

/// #### Format data size in bites to nicely readable units.
///
/// ##### Arguments
/// * `size`: Input size in bites
/// * `color`: Output color for rich markup, defaults to "cyan"
/// * `return`: String of data size wrapped in color markup
pub fn bites2str(size: u64, styler: &Styler) -> String {
    let fsize = size as f64;
    let unit: &str;
    let unit_size: f32;

    let base: f64 = 1000.;

    if fsize < base {
        unit = "B";
        unit_size = fsize as f32;
    } else if fsize < base.powf(2.) {
        unit = "KB";
        unit_size = (fsize / base) as f32;
    } else if fsize < base.powf(3.) {
        unit = "MB";
        unit_size = (fsize / base.powf(2.)) as f32;
    } else if fsize < base.powf(4.) {
        unit = "GB";
        unit_size = (fsize / base.powf(3.)) as f32;
    } else if fsize < base.powf(5.) {
        unit = "TB";
        unit_size = (fsize / base.powf(4.)) as f32;
    } else {
        unit = "PB";
        unit_size = (fsize / base.powf(5.)) as f32;
    }
    return styler.style(format!("{:7.2} {}", unit_size, unit).as_str());
}

#[cfg(test)]
mod test_styler {

    use super::Styler;

    #[test]
    fn do_nothing() {
        let styler = Styler::build("", "", false, false, "").unwrap();
        assert_eq!("some_string", styler.style("some_string"));
    }

    #[test]
    fn fg_color() {
        let styler = Styler::build("cyan", "", false, false, "").unwrap();
        assert_eq!(
            "\u{1b}[96msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn bg_color() {
        let styler = Styler::build("", "yellow", false, false, "").unwrap();
        assert_eq!(
            "\u{1b}[43msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn bold() {
        let styler = Styler::build("", "", true, false, "").unwrap();
        assert_eq!("\u{1b}[1msome_string\u{1b}[0m", styler.style("some_string"));
    }

    #[test]
    fn undrline() {
        let styler = Styler::build("", "", false, true, "").unwrap();
        assert_eq!("\u{1b}[4msome_string\u{1b}[0m", styler.style("some_string"));
    }

    #[test]
    fn all_in_style() {
        let styler = Styler::build("red", "green", true, true, "").unwrap();
        assert_eq!(
            "\u{1b}[1;4;31;42msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn regex() {
        let styler = Styler::build("red", "green", true, true, "me_st").unwrap();
        assert_eq!(
            "so\u{1b}[1;4;31;42mme_st\u{1b}[0mring",
            styler.style("some_string")
        );
    }
}

#[cfg(test)]
mod test_bytes2str {

    use super::{bites2str, Styler};

    #[test]
    fn test_bytes2str() {
        let inputs = [
            5u64,
            1024u64,
            100000024u64,
            100000000024u64,
            1000000000024u64,
            1000000000000024u64,
            14000000000000000024u64,
        ];
        let exp_results = [
            "   5.00 B",
            "   1.02 KB",
            " 100.00 MB",
            " 100.00 GB",
            "   1.00 TB",
            "   1.00 PB",
            "14000.00 PB",
        ];

        let styler = Styler::build("", "", false, false, "").unwrap();

        for it in inputs.iter().zip(exp_results.iter()) {
            let (input, exp_result) = it;
            assert_eq!(exp_result.to_string(), bites2str(*input, &styler))
        }
    }
}
