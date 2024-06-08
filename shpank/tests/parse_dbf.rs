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
fn file_lengths(#[files("data/*.dbf")] path: PathBuf) {
    let expected = std::fs::File::open(&path)
        .unwrap()
        .metadata()
        .unwrap()
        .len() as usize;

    let header = parser(&path).parse_dbase_header().unwrap();

    assert_eq!(header.dbase_num_bytes_total(), expected);
}

#[rstest]
fn one_record(#[files("data/*.dbf")] path: PathBuf) {
    let mut parser = Parser::new(path).unwrap();
    let header = parser.parse_dbase_header().unwrap();
    dbg!(&header);

    let record = parser.parse_dbase_record(&header).unwrap();
    dbg!(&record);
}

#[rstest]
fn parse_file(#[files("data/*.dbf")] path: PathBuf) {
    let _parsed = Parser::parse_dbf_file(path).unwrap();
}

#[rstest]
fn parse_file_in_mem(#[files("data/*.dbf")] path: PathBuf) {
    let buf = std::fs::read(path).unwrap();
    let _parsed = Parser::parse_dbf_buffer(&buf).unwrap();
}
