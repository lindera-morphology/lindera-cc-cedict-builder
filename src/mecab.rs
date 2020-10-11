use regex::Regex;
use std::cmp::max;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufRead};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

use super::error::ParsingError;

fn read_raw_file<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, ParsingError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

struct Word {
    word: String,
    cost: i64,
    traditional: String,
    simplified: String,
    pinyin: String,
    definition: String,
}

impl Word {
    fn to_mecab(&self) -> String {
        format!(
            "{},0,0,{},*,*,*,*,{},{},{},{}\n",
            self.word, self.cost, self.pinyin, self.traditional, self.simplified, self.definition,
        )
    }
}

struct Mecab {
    words: Vec<Word>,
}

impl Mecab {
    fn from_raw(raw: io::Lines<io::BufReader<File>>) -> Mecab {
        let mut words = Vec::new();
        let re = Regex::new(r"^(.*?) (.*?) \[(.*?)\] /(.*?)$").unwrap();

        for line in raw {
            if let Ok(line) = line {
                if let Some(f) = line.chars().nth(0) {
                    if f == '%' {
                        continue;
                    }
                } else {
                    continue;
                }

                let caps = re.captures(&line[..]);
                if let Some(caps) = caps {
                    let (traditional, simplified, pinyin, definition) =
                        (&caps[1], &caps[2], &caps[3], &caps[4]);
                    let cost = max(
                        -36000,
                        (-400f64 * (traditional.graphemes(true).count() as f64).powf(1.5)) as i64,
                    );

                    words.push(Word {
                        word: traditional.to_owned(),
                        cost,
                        traditional: traditional.to_owned(),
                        simplified: simplified.to_owned(),
                        pinyin: pinyin.to_owned(),
                        definition: definition.to_owned(),
                    });
                    if traditional != simplified {
                        words.push(Word {
                            word: simplified.to_owned(),
                            cost,
                            traditional: traditional.to_owned(),
                            simplified: simplified.to_owned(),
                            pinyin: pinyin.to_owned(),
                            definition: definition.to_owned(),
                        });
                    }
                }
            }
        }

        Mecab {
            words
        }
    }

    fn to_csv(&self, output_dir: &str) -> Result<(), ParsingError> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("cedict.csv")),
        )?);
        for word in &self.words {
            wtr.write_all(word.to_mecab().as_bytes())?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn to_matrix(&self, output_dir: &str) -> Result<(), ParsingError> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("matrix.def")),
        )?);
        wtr.write_all("1 1\n\
        0 0 0".as_bytes())?;
        wtr.flush()?;
        Ok(())
    }
}

pub fn build_mecab(input_dir: &str, output_dir: &str) -> Result<(), String> {
    fs::create_dir_all(&output_dir).unwrap_or_default();
    let raw = read_raw_file(input_dir)?;
    let mecab = Mecab::from_raw(raw);
    mecab.to_csv(output_dir)?;
    mecab.to_matrix(output_dir)?;
    Ok(())
}
