use rstest::rstest;
use shpank::parse::Parser;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

fn parser(path: &Path) -> Parser<BufReader<File>> {
    Parser::new(&path).unwrap()
}

#[rstest]
fn file_lengths(#[files("data/*.shp")] path: PathBuf) {
    let expected = std::fs::File::open(&path)
        .unwrap()
        .metadata()
        .unwrap()
        .len() as usize;

    let header = parser(&path).parse_header().unwrap();

    assert_eq!(header.file_length.num_bytes(), expected);
}

#[rstest]
fn one_record(#[files("data/*.shp")] path: PathBuf) {
    let mut parser = Parser::new(&path).unwrap();
    let _header = parser.parse_header().unwrap();
    let _record = parser.parse_record().unwrap();
}

#[rstest]
fn parse_file(#[files("data/*.shp")] path: PathBuf) {
    let _parsed = Parser::parse_file(&path).unwrap();
}
