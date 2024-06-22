use std::path::PathBuf;

use argh::FromArgs;
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

    /// only keep objects which have a non-empty name
    #[argh(switch)]
    named: bool,
}

fn main() {
    let Args { shp, dbf, named } = argh::from_env();

    println!("Creating objects from {shp:?} and {dbf:?}");
    let spatial = if named {
        Spatial::new(&shp, &dbf).unwrap().into_named_objects()
    } else {
        Spatial::new(&shp, &dbf).unwrap().into_objects()
    };

    println!("Spatial files parse OK- {} records", spatial.len());

    std::fs::write("objects.txt", format!("{spatial:#?}").as_bytes()).unwrap();
}
