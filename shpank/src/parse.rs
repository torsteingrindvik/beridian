use std::{
    fs::File,
    io::{self, BufReader, Read},
    mem::size_of,
    ops::Range,
    path::Path,
};

use thiserror::Error;

use crate::shape::{self, Point, Polygon, Shape, ShapeType};

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO")]
    IO(#[from] io::Error),

    #[error("Unexpected data: {0}")]
    UnexpectedData(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl TryFrom<i32> for ShapeType {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            v if v == ShapeType::Null as i32 => ShapeType::Null,
            v if v == ShapeType::Point as i32 => ShapeType::Point,
            v if v == ShapeType::PolyLine as i32 => ShapeType::PolyLine,
            v if v == ShapeType::Polygon as i32 => ShapeType::Polygon,
            v if v == ShapeType::MultiPoint as i32 => ShapeType::MultiPoint,
            v if v == ShapeType::PointZ as i32 => ShapeType::PointZ,
            v if v == ShapeType::PolylineZ as i32 => ShapeType::PolylineZ,
            v if v == ShapeType::PolygonZ as i32 => ShapeType::PolygonZ,
            v if v == ShapeType::MultiPointZ as i32 => ShapeType::MultiPointZ,
            v if v == ShapeType::PointM as i32 => ShapeType::PointM,
            v if v == ShapeType::PolylineM as i32 => ShapeType::PolylineM,
            v if v == ShapeType::PolygonM as i32 => ShapeType::PolygonM,
            v if v == ShapeType::MultiPointM as i32 => ShapeType::MultiPointM,
            v if v == ShapeType::MultiPatch as i32 => ShapeType::MultiPatch,
            _ => {
                return Err(Error::UnexpectedData(format!(
                    "The number `{value}` does not correspond to a shape type"
                )))
            }
        })
    }
}

#[derive(Debug)]
pub struct MinimumBoundingRectangle {
    x: Range<f64>,
    y: Range<f64>,
}

#[derive(Debug)]
pub struct ShpLength(i32);

impl ShpLength {
    /// Convert a Shapefile length to number of bytes.
    /// Use when lengths are expressed in # of 16-bit words.
    pub fn num_bytes(&self) -> usize {
        self.0 as usize * 2
    }
}

// See https://en.wikipedia.org/wiki/Shapefile#Shapefile_headers
#[derive(Debug)]
pub struct ShpHeader {
    pub file_code: i32,

    /// In 16-bit words
    pub file_length: ShpLength,

    pub version: i32,
    pub shape_type: ShapeType,
    pub mbr: MinimumBoundingRectangle,
    pub z_range: Range<f64>,
    pub m_range: Range<f64>,
}

#[derive(Debug, Default)]
struct ShpHeaderUnparsed {
    file_code: [u8; size_of::<i32>()],

    unused: [u8; 5 * size_of::<i32>()],

    file_length: [u8; size_of::<i32>()],
    version: [u8; size_of::<i32>()],
    shape_type: [u8; size_of::<i32>()],

    min_x: [u8; size_of::<f64>()],
    min_y: [u8; size_of::<f64>()],
    max_x: [u8; size_of::<f64>()],
    max_y: [u8; size_of::<f64>()],

    min_z: [u8; size_of::<f64>()],
    max_z: [u8; size_of::<f64>()],

    min_m: [u8; size_of::<f64>()],
    max_m: [u8; size_of::<f64>()],
}

impl ShpHeader {
    const FILE_CODE: i32 = 0x0000270a;
}

// #[derive(Debug, Default)]
// struct ShpRecordHeaderUnparsed {
//     number: [u8; size_of::<u32>()],
//     length: [u8; size_of::<u32>()],
// }

#[derive(Debug)]
struct ShpRecordHeader {
    /// Starting at 1
    record_number: i32,

    /// In 16-bit words.
    /// This is not including the record header.
    content_length: ShpLength,
}

// #[derive(Debug, Default)]
// struct ShpRecordUnparsed {
//     shape_type: [u8; size_of::<i32>()],
//     contents: Vec<u8>,
// }

#[derive(Debug)]
struct ShpRecord {
    shape: Shape,
}

pub struct Parser<R> {
    bytes_read: usize,
    reader: R,
}

impl<R> Parser<R> {
    pub fn num_bytes_read(&self) -> usize {
        self.bytes_read
    }
}

impl Parser<BufReader<File>> {
    pub fn new<P: AsRef<Path>>(shp_path: P) -> Result<Self> {
        let f = std::fs::File::open(shp_path.as_ref())?;
        Ok(Self::with_reader(BufReader::new(f)))
    }
}

impl<R> Parser<R>
where
    R: io::Read,
{
    pub fn with_reader(reader: R) -> Self {
        Self {
            bytes_read: 0,
            reader,
        }
    }

    // pub fn new<P: AsRef<Path>>(shp_path: P) -> Result<Self> {
    //     let f = std::fs::File::open(shp_path.as_ref())?;
    //     Ok(Self::with_reader(BufReader::new(f)))
    // }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(buf)?;
        self.bytes_read += buf.len();

        Ok(())
    }

    fn consume_4(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    fn consume_8(&mut self) -> Result<[u8; 8]> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    // Data should be the size of four floats.
    fn parse_mbr(&self, data: &[u8]) -> Result<MinimumBoundingRectangle> {
        let mut buf = [0u8; size_of::<f64>()];
        let mut reader = BufReader::new(data);

        reader.read_exact(&mut buf)?;
        let min_x = f64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let min_y = f64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let max_x = f64::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let max_y = f64::from_le_bytes(buf);

        Ok(MinimumBoundingRectangle {
            x: min_x..max_x,
            y: min_y..max_y,
        })
    }

    pub fn consume_and_parse_mbr(&mut self) -> Result<MinimumBoundingRectangle> {
        let mut buf = Vec::with_capacity(4 * size_of::<f64>());

        buf.extend_from_slice(&self.consume_8()?);
        buf.extend_from_slice(&self.consume_8()?);
        buf.extend_from_slice(&self.consume_8()?);
        buf.extend_from_slice(&self.consume_8()?);

        self.parse_mbr(&buf)
    }

    /// From "ESRI Shapefile Technical Description" doubles
    /// are 8 bytes, little endian.
    pub fn consume_and_parse_double(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.consume_8()?))
    }

    pub fn consume_and_parse_point(&mut self) -> Result<shape::Point> {
        Ok(Point {
            x: self.consume_and_parse_double()?,
            y: self.consume_and_parse_double()?,
        })
    }

    // fn parse_shape_type(&self, data: [u8; size_of::<i32>()]) -> Result<ShapeType> {
    //     let shape_type = i32::from_le_bytes(data);
    //     shape_type.try_into()
    // }

    fn consume_and_parse_shape_type(&mut self) -> Result<ShapeType> {
        self.consume_and_parse_i32_le()?.try_into()
    }

    fn consume_and_parse_i32_le(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.consume_4()?))
    }

    fn consume_and_parse_i32_be(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.consume_4()?))
    }

    /// From "ESRI Shapefile Technical Description" integers
    /// are four byte little endian numbers.
    pub fn consume_and_parse_integer(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.consume_4()?))
    }

    pub fn consume_and_parse_length(&mut self) -> Result<ShpLength> {
        self.consume_and_parse_i32_be().map(ShpLength)
    }

    pub fn consume_and_parse_record_number(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.consume_4()?))
    }

    pub fn parse_header(&mut self) -> Result<ShpHeader> {
        let ShpHeaderUnparsed {
            mut file_code,
            mut unused,
            mut file_length,
            mut version,
            mut shape_type,
            mut min_x,
            mut min_y,
            mut max_x,
            mut max_y,
            mut min_z,
            mut max_z,
            mut min_m,
            mut max_m,
        } = ShpHeaderUnparsed::default();

        // File code: 4 be bytes
        self.read_exact(&mut file_code)?;
        let file_code = i32::from_be_bytes(file_code);
        if file_code != ShpHeader::FILE_CODE {
            return Err(Error::UnexpectedData(format!(
                "Bad file code, expected `0x{:08x?}` got `0x{:08x?}`",
                ShpHeader::FILE_CODE,
                file_code
            )));
        }

        // Unused; 5 u32
        self.read_exact(&mut unused)?;

        // File length
        // self.read_exact(&mut file_length)?;
        // let file_length = i32::from_be_bytes(file_length);
        let file_length = self.consume_and_parse_length()?;

        // Version
        self.read_exact(&mut version)?;
        let version = i32::from_le_bytes(version);

        // Shape type
        let shape_type = self.consume_and_parse_shape_type()?;
        let mbr = self.consume_and_parse_mbr()?;

        self.read_exact(&mut min_z)?;
        let min_z = f64::from_le_bytes(min_z);

        self.read_exact(&mut max_z)?;
        let max_z = f64::from_le_bytes(max_z);

        self.read_exact(&mut min_m)?;
        let min_m = f64::from_le_bytes(min_m);

        self.read_exact(&mut max_m)?;
        let max_m = f64::from_le_bytes(max_m);

        let shp_header = ShpHeader {
            file_code,
            file_length,
            version,
            shape_type,
            mbr,
            z_range: min_z..max_z,
            m_range: min_m..max_m,
        };

        Ok(shp_header)
    }

    pub fn parse_record_header(&mut self) -> Result<ShpRecordHeader> {
        Ok(ShpRecordHeader {
            record_number: self.consume_and_parse_record_number()?,
            content_length: self.consume_and_parse_length()?,
        })
    }

    // pub fn parse_record_shape(&mut self, record_header: &ShpRecordHeader) -> Result<Shape> {}

    fn parse_null(&mut self) -> Shape {
        Shape::Null
    }

    fn parse_polygon(&mut self) -> Result<Shape> {
        let mbr = self.consume_and_parse_mbr()?;

        let num_parts = self.consume_and_parse_integer()?;
        let num_points = self.consume_and_parse_integer()?;

        let mut parts = Vec::with_capacity(4 * num_parts as usize);

        for _ in 0..num_parts {
            let part_idx = self.consume_and_parse_integer()?;
            dbg!(&part_idx);
            parts.push(part_idx);
        }

        let mut points = Vec::with_capacity(size_of::<Point>() * num_points as usize);
        for _ in 0..num_points {
            let point = self.consume_and_parse_point()?;
            dbg!(&point);
            points.push(point);
        }

        Ok(Shape::Polygon(Polygon { parts, points }))
    }

    pub fn parse_record(&mut self) -> Result<ShpRecord> {
        let record_header = self.parse_record_header()?;
        dbg!(&record_header);

        let num_bytes_parsed_before_record = self.num_bytes_read();
        let num_bytes_required_for_record = record_header.content_length.num_bytes();

        // Shape always comes first
        let shape_type = self.consume_and_parse_shape_type()?;

        let shape = match shape_type {
            ShapeType::Null => self.parse_null(),
            ShapeType::Point => unimplemented!(),
            ShapeType::PolyLine => unimplemented!(),
            ShapeType::Polygon => self.parse_polygon()?,
            ShapeType::MultiPoint => unimplemented!(),
            ShapeType::PointZ => unimplemented!(),
            ShapeType::PolylineZ => unimplemented!(),
            ShapeType::PolygonZ => unimplemented!(),
            ShapeType::MultiPointZ => unimplemented!(),
            ShapeType::PointM => unimplemented!(),
            ShapeType::PolylineM => unimplemented!(),
            ShapeType::PolygonM => unimplemented!(),
            ShapeType::MultiPointM => unimplemented!(),
            ShapeType::MultiPatch => unimplemented!(),
        };

        let read_for_record = self.num_bytes_read() - num_bytes_parsed_before_record;
        assert_eq!(num_bytes_required_for_record, read_for_record);
        // let mut record_parser = record_header.content_parser()?;

        // debug_assert_ne!(record_contents.len(), 0);
        // self.read_exact(&mut record_contents)?;

        // let shape = Shape::parse(&shape_type, &record_contents)?;

        Ok(ShpRecord { shape })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    enum TestFiles {
        BuildingsA,
        LanduseA,
        NaturalA,
        Natural,
        PlacesA,
        Places,
        PofwA,
        Pofw,
        PoisA,
        Pois,
        Railways,
        Roads,
        TrafficA,
        Traffic,
        TransportA,
        Transport,
        WaterA,
        Waterways,
    }

    impl TestFiles {
        fn all() -> [Self; 18] {
            [
                TestFiles::BuildingsA,
                TestFiles::LanduseA,
                TestFiles::NaturalA,
                TestFiles::Natural,
                TestFiles::PlacesA,
                TestFiles::Places,
                TestFiles::PofwA,
                TestFiles::Pofw,
                TestFiles::PoisA,
                TestFiles::Pois,
                TestFiles::Railways,
                TestFiles::Roads,
                TestFiles::TrafficA,
                TestFiles::Traffic,
                TestFiles::TransportA,
                TestFiles::Transport,
                TestFiles::WaterA,
                TestFiles::Waterways,
            ]
        }

        fn path(&self) -> String {
            let s = match self {
                TestFiles::BuildingsA => "buildings_a",
                TestFiles::LanduseA => "landuse_a",
                TestFiles::NaturalA => "natural_a",
                TestFiles::Natural => "natural",
                TestFiles::PlacesA => "places_a",
                TestFiles::Places => "places",
                TestFiles::PofwA => "pofw_a",
                TestFiles::Pofw => "pofw",
                TestFiles::PoisA => "pois_a",
                TestFiles::Pois => "pois",
                TestFiles::Railways => "railways",
                TestFiles::Roads => "roads",
                TestFiles::TrafficA => "traffic_a",
                TestFiles::Traffic => "traffic",
                TestFiles::TransportA => "transport_a",
                TestFiles::Transport => "transport",
                TestFiles::WaterA => "water_a",
                TestFiles::Waterways => "waterways",
            };

            format!("{}/data/gis_osm_{s}_free_1.shp", env!("CARGO_MANIFEST_DIR"))
        }

        fn all_paths() -> Vec<PathBuf> {
            TestFiles::all()
                .iter()
                .map(TestFiles::path)
                .map(PathBuf::from)
                .collect()
        }
    }

    #[test]
    pub fn header() {
        for test_file_path in TestFiles::all_paths() {
            dbg!(&test_file_path);
            Parser::new(test_file_path).unwrap().parse_header().unwrap();
        }
    }

    #[test]
    pub fn file_lengths() {
        for test_file_path in TestFiles::all_paths() {
            let expected = std::fs::File::open(&test_file_path)
                .unwrap()
                .metadata()
                .unwrap()
                .len() as usize;

            let header = Parser::new(&test_file_path)
                .unwrap()
                .parse_header()
                .unwrap();
            dbg!(&test_file_path, &header);

            assert_eq!(header.file_length.num_bytes(), expected);
        }
    }

    #[test]
    pub fn one_record() {
        for test_file_path in TestFiles::all_paths() {
            let mut parser = Parser::new(test_file_path).unwrap();
            let header = parser.parse_header().unwrap();
            let record = parser.parse_record().unwrap();
        }
    }
}
