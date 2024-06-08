use rstest::rstest;
use shpank::parse::Parser;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

fn parser(path: &Path) -> Parser<BufReader<File>> {
    Parser::new(path).unwrap()
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
    let mut parser = Parser::new(path).unwrap();
    let header = parser.parse_header().unwrap();
    dbg!(&header);

    let record = parser.parse_record().unwrap();
    dbg!(&record);
}

#[rstest]
fn parse_file(#[files("data/*.shp")] path: PathBuf) {
    let _parsed = Parser::parse_shp_file(path).unwrap();
}

#[rstest]
fn parse_file_in_mem(#[files("data/*.shp")] path: PathBuf) {
    let buf = std::fs::read(path).unwrap();
    let _parsed = Parser::parse_shp_buffer(&buf).unwrap();
}
