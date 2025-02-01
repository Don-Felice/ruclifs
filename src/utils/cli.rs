use anyhow::{anyhow, Result};
use regex::Regex;
use std::io;

pub const INDENT: &str = "    ";

pub fn print_line(text: &str) {
    println!("{:―^50}", text);
}

#[derive(PartialEq)]
pub enum AnsiColor {
    Cyan,
    Green,
    Gray,
    Red,
    Yellow,
    Default,
}

impl AnsiColor {
    fn apply_fg(&self, style: &mut Vec<&str>) {
        match self {
            AnsiColor::Cyan => style.push("96"),
            AnsiColor::Green => style.push("32"),
            AnsiColor::Gray => style.push("90"),
            AnsiColor::Red => style.push("31"),
            AnsiColor::Yellow => style.push("33"),
            AnsiColor::Default => (),
        }
    }

    fn apply_bg(&self, style: &mut Vec<&str>) {
        match self {
            AnsiColor::Cyan => style.push("46"),
            AnsiColor::Green => style.push("42"),
            AnsiColor::Gray => style.push("100"),
            AnsiColor::Red => style.push("41"),
            AnsiColor::Yellow => style.push("43"),
            AnsiColor::Default => (),
        }
    }
}

pub struct Styler {
    style_seq: String,
    reset_seq: String,
    regex: Option<Regex>,
}
impl Styler {
    pub fn build(
        color_fg: &AnsiColor,
        color_bg: &AnsiColor,
        bold: bool,
        underline: bool,
        pattern: &str,
    ) -> Result<Styler> {
        // do nothing if no options are chosen
        if color_fg == &AnsiColor::Default
            && color_bg == &AnsiColor::Default
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

        // set foreground
        color_fg.apply_fg(&mut style);

        // set background
        color_bg.apply_bg(&mut style);

        let style_str = style.join(";");
        let style_seq = format!("\x1b[{}m", style_str);

        // get regex
        if pattern != "" {
            let style_regex = match Regex::new(pattern) {
                Ok(r) => r,
                Err(err) => {
                    return Err(anyhow!(
                        "Error when trying to compile a regex from '{:?}':\n{}",
                        pattern,
                        err
                    ));
                }
            };
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
                .replace_all(text, format!("{}$0{}", &self.style_seq, &self.reset_seq))
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

    use super::{AnsiColor, Styler};

    #[test]
    fn do_nothing() {
        let styler =
            Styler::build(&AnsiColor::Default, &AnsiColor::Default, false, false, "")
                .unwrap();
        assert_eq!("some_string", styler.style("some_string"));
    }

    #[test]
    fn fg_color() {
        let styler =
            Styler::build(&AnsiColor::Cyan, &AnsiColor::Default, false, false, "")
                .unwrap();
        assert_eq!(
            "\u{1b}[96msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn bg_color() {
        let styler =
            Styler::build(&AnsiColor::Default, &AnsiColor::Yellow, false, false, "")
                .unwrap();
        assert_eq!(
            "\u{1b}[43msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn bold() {
        let styler =
            Styler::build(&AnsiColor::Default, &AnsiColor::Default, true, false, "")
                .unwrap();
        assert_eq!("\u{1b}[1msome_string\u{1b}[0m", styler.style("some_string"));
    }

    #[test]
    fn undrline() {
        let styler =
            Styler::build(&AnsiColor::Default, &AnsiColor::Default, false, true, "")
                .unwrap();
        assert_eq!("\u{1b}[4msome_string\u{1b}[0m", styler.style("some_string"));
    }

    #[test]
    fn all_in_style() {
        let styler =
            Styler::build(&AnsiColor::Red, &AnsiColor::Green, true, true, "").unwrap();
        assert_eq!(
            "\u{1b}[1;4;31;42msome_string\u{1b}[0m",
            styler.style("some_string")
        );
    }

    #[test]
    fn regex() {
        let styler =
            Styler::build(&AnsiColor::Red, &AnsiColor::Green, true, true, "me_st")
                .unwrap();
        assert_eq!(
            "so\u{1b}[1;4;31;42mme_st\u{1b}[0mring",
            styler.style("some_string")
        );
    }
}

#[cfg(test)]
mod test_bytes2str {

    use super::{bites2str, AnsiColor, Styler};

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

        let styler =
            Styler::build(&AnsiColor::Default, &AnsiColor::Default, false, false, "")
                .unwrap();

        for it in inputs.iter().zip(exp_results.iter()) {
            let (input, exp_result) = it;
            assert_eq!(exp_result.to_string(), bites2str(*input, &styler))
        }
    }
}
