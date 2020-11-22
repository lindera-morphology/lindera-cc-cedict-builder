use super::error::Result;
use super::mecab::{read_raw_file, Mecab};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::result::Result as stdResult;
use std::str::FromStr;

use byteorder::{LittleEndian, WriteBytesExt};
use lindera_core::core::{
    character_definition::CharacterDefinitions,
    word_entry::{WordEntry, WordId},
};
use lindera_ipadic_builder::{parse_unk, CharacterDefinitionsBuilder};
use yada::builder::DoubleArrayBuilder;

struct Lindera<'a> {
    mecab: &'a Mecab<'a>,
}

impl<'a> Lindera<'a> {
    fn from_mecab(mecab: &'a Mecab) -> Lindera<'a> {
        Lindera { mecab }
    }

    fn build_dict(&self, output_dir: &str) -> Result<()> {
        let mut word_entry_map: BTreeMap<String, Vec<WordEntry>> = BTreeMap::new();

        for (word_id, word) in self.mecab.words.iter().enumerate() {
            if word.word_cost < i16::MIN as i32 {
                println!(
                    "{}'s cost is {}, less than i16::MIN",
                    word.word, word.word_cost
                );
                continue;
            }
            word_entry_map
                .entry(word.traditional.to_string())
                .or_insert_with(Vec::new)
                .push(WordEntry {
                    word_id: WordId(word_id as u32, true),
                    word_cost: word.word_cost as i16,
                    cost_id: 0,
                });
        }

        let mut wtr_words = io::BufWriter::new(File::create(
            Path::new(output_dir).join(Path::new("dict.words")),
        )?);
        let mut wtr_words_idx = io::BufWriter::new(File::create(
            Path::new(output_dir).join(Path::new("dict.wordsidx")),
        )?);
        let mut words_buffer = Vec::new();
        for word in self.mecab.words.iter() {
            let word = vec![word.pinyin, word.definition];
            let offset = words_buffer.len();
            wtr_words_idx.write_u32::<LittleEndian>(offset as u32)?;
            bincode::serialize_into(&mut words_buffer, &word).unwrap();
        }

        wtr_words.write_all(&words_buffer[..])?;
        wtr_words.flush()?;
        wtr_words_idx.flush()?;

        let mut id: u32 = 0u32;

        println!("building da");
        let mut wtr_da = io::BufWriter::new(
            File::create(Path::new(output_dir).join(Path::new("dict.da"))).unwrap(),
        );

        let mut keyset = Vec::new();
        let mut lastlen: u32 = 0;
        for (key, word_entries) in &word_entry_map {
            let len = word_entries.len() as u32;
            assert!(
                len < (1 << 5),
                format!("{} is {} length. Too long. [{}]", key, len, (1 << 5))
            );
            let val = (id << 5) | len;
            keyset.push((key.as_bytes(), val));
            id += len;
            lastlen += len;
        }
        let da_bytes = DoubleArrayBuilder::build(&keyset);
        assert!(da_bytes.is_some(), "DoubleArray build error. ");
        wtr_da.write_all(&da_bytes.unwrap()[..])?;
        println!("Last len is {}", lastlen);

        println!("building values");
        let mut wtr_vals = io::BufWriter::new(
            File::create(Path::new(output_dir).join(Path::new("dict.vals"))).unwrap(),
        );
        for word_entries in word_entry_map.values() {
            for word_entry in word_entries {
                word_entry.serialize(&mut wtr_vals)?;
            }
        }
        wtr_vals.flush().unwrap();
        Ok(())
    }

    fn build_cost_matrix(&self, output_dir: &str) -> Result<()> {
        let mut lines = Vec::new();
        let matrix_def = std::str::from_utf8(self.mecab.matrix)?;
        for line in matrix_def.lines() {
            let fields: Vec<i32> = line
                .split_whitespace()
                .map(i32::from_str)
                .collect::<stdResult<_, _>>()?;
            lines.push(fields);
        }
        let mut lines_it = lines.into_iter();
        let header = lines_it.next().unwrap();
        let forward_size = header[0] as u32;
        let backward_size = header[1] as u32;
        let len = 2 + (forward_size * backward_size) as usize;
        let mut costs = vec![i16::max_value(); len];
        costs[0] = forward_size as i16;
        costs[1] = backward_size as i16;
        for fields in lines_it {
            let forward_id = fields[0] as u32;
            let backward_id = fields[1] as u32;
            let cost = fields[2] as u16;
            costs[2 + (backward_id + forward_id * backward_size) as usize] = cost as i16;
        }

        let mut wtr = io::BufWriter::new(File::create(
            Path::new(output_dir).join(Path::new("matrix.mtx")),
        )?);
        for cost in costs {
            wtr.write_i16::<LittleEndian>(cost)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn build_chardef(&self, output_dir: &str) -> Result<CharacterDefinitions> {
        let mut char_definitions_builder = CharacterDefinitionsBuilder::default();
        let char_def = std::str::from_utf8(self.mecab.char_def)?;
        char_definitions_builder.parse(&char_def.to_string())?;
        let char_definitions = char_definitions_builder.build();
        let mut wtr_chardef = io::BufWriter::new(File::create(
            Path::new(output_dir).join(Path::new("char_def.bin")),
        )?);
        bincode::serialize_into(&mut wtr_chardef, &char_definitions)?;
        wtr_chardef.flush()?;
        Ok(char_definitions)
    }

    fn build_unk(&self, chardef: &CharacterDefinitions, output_dir: &str) -> Result<()> {
        let unk_data = std::str::from_utf8(self.mecab.unk_def)?;
        let unknown_dictionary = parse_unk(&chardef.categories(), &unk_data.to_string())?;
        let mut wtr_unk = io::BufWriter::new(File::create(
            Path::new(output_dir).join(Path::new("unk.bin")),
        )?);
        bincode::serialize_into(&mut wtr_unk, &unknown_dictionary)?;
        wtr_unk.flush()?;
        Ok(())
    }
}

pub fn build(input_dir: &str, output_dir: &str) -> Result<()> {
    fs::create_dir_all(&output_dir).unwrap_or_default();

    let raw = read_raw_file(input_dir)?;
    let mecab = Mecab::from_raw(&raw);
    let lindera = Lindera::from_mecab(&mecab);
    let char_def = lindera.build_chardef(output_dir)?;
    lindera.build_unk(&char_def, output_dir)?;
    lindera.build_dict(output_dir)?;
    lindera.build_cost_matrix(output_dir)?;

    Ok(())
}
