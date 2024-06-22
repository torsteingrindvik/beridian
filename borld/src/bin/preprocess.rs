use std::{path::PathBuf, time::Instant};

use argh::FromArgs;
use borld::preprocess::Object;
use shpank::spatial::Spatial;

#[derive(Debug, FromArgs)]
/// Parse a .shp- and .dbf file pair then convert to objects.
struct Args {
    /// path to input Shapefile
    #[argh(positional)]
    shp: PathBuf,

    /// path to input dBASE file
    #[argh(positional)]
    dbf: PathBuf,

    /// output file path, uses shp file stem with ".borld" ending if not given
    #[argh(option)]
    out: Option<PathBuf>,
}

fn main() {
    let Args { shp, dbf, out } = argh::from_env();

    let out = out.unwrap_or_else(|| shp.with_extension("borld"));

    let start = Instant::now();
    println!("Creating shp objects from {shp:?} and {dbf:?}");

    let objects = Spatial::new(&shp, &dbf).unwrap().into_objects();

    println!(
        "shp objects ok ({:.2}s), processing..",
        start.elapsed().as_secs_f32()
    );

    let objects: Vec<Object> = objects.into_iter().map(Into::into).collect();

    println!(
        "preprocessed objects ok (total: {:.2}s), writing to output {out:?}",
        start.elapsed().as_secs_f32()
    );

    let bytes = bincode::serialize(&objects).unwrap();
    std::fs::write(out, bytes).unwrap();
}
