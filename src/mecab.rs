use regex::Regex;
use std::cmp::max;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufRead};
use std::path::Path;
use std::result::Result as stdResult;
use unicode_segmentation::UnicodeSegmentation;

use super::error::Result;

const UNK_DEF: &[u8] = include_bytes!("./static/unk.def");
const CHAR_DEF: &[u8] = include_bytes!("./static/char.def");
const DICRC: &[u8] = include_bytes!("./static/dicrc");
const MATRIX: &[u8] = include_bytes!("./static/matrix.def");

pub fn read_raw_file<P>(filename: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file)
        .lines()
        .filter(stdResult::is_ok)
        .map(|line| line.unwrap())
        .collect())
}

pub struct Word<'a> {
    pub word: &'a str,
    pub left_id: u32,
    pub right_id: u32,
    pub word_cost: i32,
    pub traditional: &'a str,
    pub simplified: &'a str,
    pub pinyin: &'a str,
    pub definition: &'a str,
}

impl<'a> Word<'a> {
    fn to_mecab(&self) -> String {
        format!(
            "{},{},{},{},*,*,*,*,{},{},{},{}\n",
            self.word,
            0,
            0,
            self.word_cost,
            self.pinyin,
            self.traditional,
            self.simplified,
            self.definition,
        )
    }
}

pub struct Matrix<'a> {
    l_size: u32,
    r_size: u32,
    l_index: u32,
    r_index: u32,
    words: &'a Vec<Word<'a>>,
}

impl<'a> Iterator for Matrix<'a> {
    type Item = (u32, u32, i32);
    fn next(&mut self) -> Option<(u32, u32, i32)> {
        if self.l_index == self.l_size {
            return None;
        }
        let lword = &self.words[self.l_index as usize];
        let rword = &self.words[self.r_index as usize];
        let cost = (lword.word_cost + rword.word_cost) / 10;
        let result = (self.l_index, self.r_index, cost);
        self.r_index += 1;
        if self.r_index == self.r_size {
            self.r_index = 0;
            self.l_index += 1;
        }
        Some(result)
    }
}

pub struct Mecab<'a> {
    pub words: Vec<Word<'a>>,
    pub unk_def: &'static [u8],
    pub char_def: &'static [u8],
    pub dicrc: &'static [u8],
    pub matrix: &'static [u8],
}

impl<'a> Mecab<'a> {
    fn new() -> Mecab<'a> {
        Mecab {
            words: Vec::new(),
            unk_def: UNK_DEF,
            char_def: CHAR_DEF,
            dicrc: DICRC,
            matrix: MATRIX,
        }
    }

    pub fn from_raw(raw: &'a [String]) -> Mecab<'a> {
        let mut mecab = Self::new();
        let re = Regex::new(r"^(.*?) (.*?) \[(.*?)\] /(.*?)$").unwrap();
        let mut id = 0;

        for line in raw {
            if let Some(f) = line.chars().next() {
                if f == '%' {
                    continue;
                }
            } else {
                continue;
            }
            let caps = re.captures(line);
            if let Some(caps) = caps {
                let (traditional, simplified, pinyin, definition) = (
                    caps.get(1).unwrap().as_str(),
                    caps.get(2).unwrap().as_str(),
                    caps.get(3).unwrap().as_str(),
                    caps.get(4).unwrap().as_str(),
                );
                let word_cost = max(
                    -36000,
                    (-400f64 * (traditional.graphemes(true).count() as f64).powf(1.5)) as i32,
                );

                mecab.words.push(Word {
                    word: traditional,
                    left_id: id,
                    right_id: id,
                    word_cost,
                    traditional,
                    simplified,
                    pinyin,
                    definition,
                });
                if traditional != simplified {
                    mecab.words.push(Word {
                        word: simplified,
                        left_id: id,
                        right_id: id,
                        word_cost,
                        traditional,
                        simplified,
                        pinyin,
                        definition,
                    });
                }
                id += 1;
            }
        }

        mecab
    }

    #[allow(dead_code)]
    pub fn matrix(&self) -> Matrix {
        let size = self.words.len() as u32;
        Matrix {
            l_size: size,
            r_size: size,
            l_index: 0,
            r_index: 0,
            words: &self.words,
        }
    }

    fn to_csv(&self, output_dir: &str) -> Result<()> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("cedict.csv")),
        )?);
        for word in &self.words {
            wtr.write_all(word.to_mecab().as_bytes())?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn to_matrix(&self, output_dir: &str) -> Result<()> {
        // let mut wtr = io::LineWriter::new(File::create(
        //     Path::new(output_dir).join(Path::new("matrix.def")),
        // )?);
        // let matrix = self.matrix();
        // write!(&mut wtr, "{} {}\n", matrix.l_size, matrix.r_size)?;
        // // the size of the matrix will be 193173 * 193173
        // let mut i: u64 = 0;
        // let all: u64 = 193173 * 193173;
        // for pair in matrix {
        //     write!(&mut wtr, "{} {} {}\n", pair.0, pair.1, pair.2)?;
        //     if i % 1000000 == 0 {
        //         println!("{}/{}",i,all);
        //         wtr.flush()?;
        //     }
        //     i+=1;
        // }
        // wtr.flush()?;
        // Ok(())

        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("matrix.def")),
        )?);
        wtr.write_all(MATRIX)?;
        wtr.flush()?;
        Ok(())
    }

    fn to_unkdef(&self, output_dir: &str) -> Result<()> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("unk.def")),
        )?);
        wtr.write_all(self.unk_def)?;
        wtr.flush()?;
        Ok(())
    }

    fn to_chardef(&self, output_dir: &str) -> Result<()> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("char.def")),
        )?);
        wtr.write_all(self.char_def)?;
        wtr.flush()?;
        Ok(())
    }

    fn to_dicrc(&self, output_dir: &str) -> Result<()> {
        let mut wtr = io::LineWriter::new(File::create(
            Path::new(output_dir).join(Path::new("dicrc")),
        )?);
        wtr.write_all(self.dicrc)?;
        wtr.flush()?;
        Ok(())
    }
}

pub fn build(input_dir: &str, output_dir: &str) -> Result<()> {
    fs::create_dir_all(&output_dir).unwrap_or_default();
    let raw = read_raw_file(input_dir)?;
    let mecab = Mecab::from_raw(&raw);
    mecab.to_csv(output_dir)?;
    mecab.to_matrix(output_dir)?;
    mecab.to_unkdef(output_dir)?;
    mecab.to_chardef(output_dir)?;
    mecab.to_dicrc(output_dir)?;
    Ok(())
}
