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
/// Parse an input .shp file then exit.
/// Can be used for speedtesting.
struct Parse {
    /// path to input file
    #[argh(option)]
    shp: PathBuf,

    /// which mode to parse in: [file|string]
    #[argh(option)]
    mode: Mode,
}

fn main() {
    let Parse { shp, mode } = argh::from_env();

    match mode {
        Mode::File => {
            let _f = Parser::parse_file(shp).unwrap();
        }
        Mode::Bytes => {
            let bytes = std::fs::read(shp).unwrap();
            let _f = Parser::parse_buffer(&bytes).unwrap();
        }
    }

    println!("OK");
}
