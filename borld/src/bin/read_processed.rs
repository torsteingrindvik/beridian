use std::{path::PathBuf, time::Instant};

use argh::FromArgs;
use borld::preprocess::{Object, Variant};

#[derive(Debug, FromArgs)]
/// read a processed .borld file and write stats
struct Args {
    /// output file path, uses shp file stem with ".borld" ending if not given
    #[argh(positional)]
    borld: PathBuf,
}

fn main() {
    let Args { borld } = argh::from_env();

    let start = Instant::now();
    println!("Reading file at {borld:?}");

    let bytes = std::fs::read(borld).unwrap();
    println!(
        "read bytes ok ({:.2}s), deserializing..",
        start.elapsed().as_secs_f32()
    );

    let objects: Vec<Object> = bincode::deserialize(&bytes).unwrap();
    println!(
        "deserialize ok (total: {:.2}s)",
        start.elapsed().as_secs_f32()
    );

    let num = objects.len();
    let with_name = objects.iter().filter(|o| o.name.is_some()).count();
    let lines = objects
        .iter()
        .filter(|o| matches!(o.variant, Variant::Line(_)))
        .count();
    let points = objects
        .iter()
        .filter(|o| matches!(o.variant, Variant::Point(_)))
        .count();
    let polygons = objects
        .iter()
        .filter(|o| matches!(o.variant, Variant::Polygon(_)))
        .count();

    println!("num objects: {num}");
    println!("named objects: {with_name}/{num}");
    println!("lines: {lines}/{num}");
    println!("points: {points}/{num}");
    println!("polygons: {polygons}/{num}");
}
