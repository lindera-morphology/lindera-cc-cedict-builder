use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};

mod error;
mod lindera;
mod mecab;

use lindera::build_lindera;
use mecab::build_mecab;

fn main() {
    let matches = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("The directory where the CEDICT source containing.")
                .value_name("INPUT_DIR")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("OUTPUT_DIR")
                .help("The directory where the CEDICT binary for Lindera is output.")
                .value_name("OUTPUT_DIR")
                .required(true)
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("mecab-cc-cedict"))
        .subcommand(SubCommand::with_name("lindera-cc-cedict"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("mecab-cc-cedict") {
        match build_mecab(
            matches.value_of("INPUT_DIR").unwrap(),
            matches.value_of("OUTPUT_DIR").unwrap(),
        ) {
            Ok(()) => println!("done"),
            Err(msg) => println!("{}", msg),
        }
    }

    if let Some(_) = matches.subcommand_matches("lindera-cc-cedict") {
        match build_lindera(
            matches.value_of("INPUT_DIR").unwrap(),
            matches.value_of("OUTPUT_DIR").unwrap(),
        ) {
            Ok(()) => println!("done"),
            Err(msg) => println!("{}", msg),
        }
    }
}
