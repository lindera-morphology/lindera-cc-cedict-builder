use regex::Regex;
use std::cmp::max;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufRead};
use std::path::Path;

use super::error::ParsingError;

fn read_raw_file<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, ParsingError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse(raw: io::Lines<io::BufReader<File>>) -> Vec<String> {
    let re = Regex::new(r"^(.*?) (.*?) \[(.*?)\] /(.*?)$").unwrap();
    raw.filter(|line| line.is_ok())
        .map(Result::unwrap)
        .map(|line| {
            if let Some(f) = line.chars().nth(0) {
                if f == '%' {
                    return None;
                }
            }
            let caps = re.captures(&line[..]);
            match caps {
                Some(caps) => {
                    let (traditional, simplified, pinyin, definition) =
                        (&caps[1], &caps[2], &caps[3], &caps[4]);
                    let cost = max(-36000, -400 * (traditional.len() as f64).powf(1.5) as i64);

                    Some(format!(
                        "{},0,0,{},*,*,*,*,{},{},{},{}\n{},0,0,{},*,*,*,*,{},{},{},{}\n",
                        traditional,
                        cost,
                        pinyin,
                        traditional,
                        simplified,
                        definition,
                        simplified,
                        cost,
                        pinyin,
                        traditional,
                        simplified,
                        definition
                    ))
                }
                None => None,
            }
        })
        .filter(|line| line.is_some())
        .map(Option::unwrap)
        .collect()
}

fn build_csv(output_dir: &str, mecab: &Vec<String>) -> Result<(), ParsingError> {
    let mut wtr_csv = io::LineWriter::new(File::create(
        Path::new(output_dir).join(Path::new("cedict.csv")),
    )?);
    for line in mecab {
        wtr_csv.write_all(line.as_bytes())?;
    }
    wtr_csv.flush()?;
    Ok(())
}

pub fn build_mecab(input_dir: &str, output_dir: &str) -> Result<(), String> {
    fs::create_dir_all(&output_dir).unwrap_or_default();
    let raw = read_raw_file(input_dir)?;
    let mecab = parse(raw);
    build_csv(output_dir, &mecab)?;
    Ok(())
}
