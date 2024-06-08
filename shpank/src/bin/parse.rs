use argh::FromArgs;
use shpank::parse::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug)]
enum Mode {
    /// Read from open file
    File,

    /// Read entire file then parse that
    Bytes,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "file" => Self::File,
            "bytes" => Self::Bytes,
            others => return Err(format!("unknown mode `{others}`")),
        })
    }
}

#[derive(Debug, FromArgs)]
/// Parse an input .shp or .dbf file then exit.
/// Can be used for speedtesting.
struct Args {
    /// path to input file
    #[argh(option)]
    file: PathBuf,

    /// which mode to parse in: [file|string]
    #[argh(option)]
    mode: Mode,
}

fn main() {
    let Args { file, mode } = argh::from_env();

    enum Ext {
        Shp,
        Dbf,
    }

    let ext = file.extension().unwrap().to_string_lossy().to_string();
    let ext = match ext.as_str() {
        "shp" => Ext::Shp,
        "dbf" => Ext::Dbf,
        _ => panic!("expected a .shp or .dbf file"),
    };

    match ext {
        Ext::Shp => {
            let f = match mode {
                Mode::File => Parser::parse_shp_file(file).unwrap(),
                Mode::Bytes => {
                    let bytes = std::fs::read(file).unwrap();
                    Parser::parse_shp_buffer(&bytes).unwrap()
                }
            };

            println!(".shp parse OK- {} records", f.records.len());
        }
        Ext::Dbf => {
            let f = match mode {
                Mode::File => Parser::parse_dbf_file(file).unwrap(),
                Mode::Bytes => {
                    let bytes = std::fs::read(file).unwrap();
                    Parser::parse_dbf_buffer(&bytes).unwrap()
                }
            };

            println!(".dbf parse OK- {} records", f.records.len());
        }
    }
}
