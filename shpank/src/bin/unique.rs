use argh::FromArgs;
use shpank::parse::Parser;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, FromArgs)]
/// Parse an input .dbf file then exit.
/// Can be used for speedtesting.
struct Parse {
    /// field to return unique entries of
    #[argh(option, default = "String::from(\"fclass\")")]
    field: String,

    /// path to input file
    #[argh(positional)]
    file: PathBuf,
}

fn main() {
    let Parse { file, field } = argh::from_env();

    let dbf = Parser::parse_dbf_file(&file).unwrap();
    let idx = dbf
        .header
        .index_of(&field)
        .expect("expected to get an index of the given field");

    println!(".dbf parse OK- field `{field}` is at index {idx}");

    let field_values = dbf
        .records
        .iter()
        .map(|entry| &entry.entries[idx])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let output = file.with_extension(format!("{field}.txt"));
    println!("writing results to {output:?}");

    std::fs::write(output, field_values.join("\n")).unwrap();
}
