use std::{
    fs::File,
    io::{self, BufReader},
    mem::size_of,
    path::Path,
};

use thiserror::Error;

use crate::shape::{
    self, Double, Integer, MinimumBoundingRectangle, Point, PolyLine, Polygon, Shape, ShapeType,
    ShpFile, ShpHeader, ShpLength, ShpRecord, ShpRecordHeader,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO")]
    IO(#[from] io::Error),

    #[error("Unexpected data: {0}")]
    UnexpectedData(String),
}

pub type Result<T> = std::result::Result<T, Error>;

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

    pub fn parse_file<P: AsRef<Path>>(shp_path: P) -> Result<ShpFile> {
        let parser = Self::new(shp_path)?;
        parser.impl_parse()
    }
}

impl<'b> Parser<&'b [u8]> {
    pub fn parse_buffer(buf: &'b [u8]) -> Result<ShpFile> {
        Self::with_reader(buf).impl_parse()
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

    fn impl_parse(mut self) -> Result<ShpFile> {
        let header = self.parse_header()?;

        let mut records = vec![];

        let goal = header.file_length.num_bytes();

        while self.num_bytes_read() < goal {
            records.push(self.parse_record()?);
        }

        assert_eq!(self.num_bytes_read(), goal);

        Ok(ShpFile { header, records })
    }

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

    fn parse_mbr(&mut self) -> Result<MinimumBoundingRectangle> {
        let min_x = self.parse_double()?;
        let min_y = self.parse_double()?;
        let max_x = self.parse_double()?;
        let max_y = self.parse_double()?;

        Ok(MinimumBoundingRectangle {
            x: min_x..max_x,
            y: min_y..max_y,
        })
    }

    /// From "ESRI Shapefile Technical Description" doubles
    /// are 8 bytes, little endian.
    fn parse_double(&mut self) -> Result<Double> {
        Ok(f64::from_le_bytes(self.consume_8()?))
    }

    fn parse_point(&mut self) -> Result<shape::Point> {
        Ok(Point {
            x: self.parse_double()?,
            y: self.parse_double()?,
        })
    }

    fn parse_shape_type(&mut self) -> Result<ShapeType> {
        self.parse_i32_le()?.try_into()
    }

    fn parse_i32_le(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.consume_4()?))
    }

    fn parse_i32_be(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.consume_4()?))
    }

    /// From "ESRI Shapefile Technical Description" integers
    /// are four byte little endian numbers.
    fn parse_integer(&mut self) -> Result<Integer> {
        Ok(i32::from_le_bytes(self.consume_4()?))
    }

    fn parse_length(&mut self) -> Result<ShpLength> {
        self.parse_i32_be().map(ShpLength)
    }

    fn parse_record_number(&mut self) -> Result<i32> {
        self.parse_i32_be()
    }

    pub fn parse_header(&mut self) -> Result<ShpHeader> {
        // File code: 4 be bytes
        let file_code = self.parse_i32_be()?;
        if file_code != ShpHeader::FILE_CODE {
            return Err(Error::UnexpectedData(format!(
                "Bad file code, expected `0x{:08x?}` got `0x{:08x?}`",
                ShpHeader::FILE_CODE,
                file_code
            )));
        }

        // Unused; 5 integers
        self.read_exact(&mut [0; 5 * size_of::<Integer>()])?;

        let file_length = self.parse_length()?;
        let version = self.parse_integer()?;
        let shape_type = self.parse_shape_type()?;
        let mbr = self.parse_mbr()?;
        let min_z = self.parse_double()?;
        let max_z = self.parse_double()?;
        let min_m = self.parse_double()?;
        let max_m = self.parse_double()?;

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

    fn parse_record_header(&mut self) -> Result<ShpRecordHeader> {
        Ok(ShpRecordHeader {
            record_number: self.parse_record_number()?,
            content_length: self.parse_length()?,
        })
    }

    fn parse_null(&mut self) -> Shape {
        Shape::Null
    }

    fn parse_polygon(&mut self) -> Result<Shape> {
        let mbr = self.parse_mbr()?;

        let num_parts = self.parse_integer()?;
        let num_points = self.parse_integer()?;

        let mut parts = Vec::with_capacity(4 * num_parts as usize);

        for _ in 0..num_parts {
            let part_idx = self.parse_integer()?;
            parts.push(part_idx);
        }

        let mut points = Vec::with_capacity(size_of::<Point>() * num_points as usize);
        for _ in 0..num_points {
            let point = self.parse_point()?;
            points.push(point);
        }

        Ok(Shape::Polygon(Polygon { parts, points, mbr }))
    }

    fn parse_polyline(&mut self) -> Result<Shape> {
        let mbr = self.parse_mbr()?;

        let num_parts = self.parse_integer()?;
        let num_points = self.parse_integer()?;

        let mut parts = Vec::with_capacity(4 * num_parts as usize);

        for _ in 0..num_parts {
            let part_idx = self.parse_integer()?;
            parts.push(part_idx);
        }

        let mut points = Vec::with_capacity(size_of::<Point>() * num_points as usize);
        for _ in 0..num_points {
            let point = self.parse_point()?;
            points.push(point);
        }

        Ok(Shape::PolyLine(PolyLine { mbr, parts, points }))
    }

    pub fn parse_record(&mut self) -> Result<ShpRecord> {
        let record_header = self.parse_record_header()?;

        let num_bytes_parsed_before_record = self.num_bytes_read();
        let num_bytes_required_for_record = record_header.content_length.num_bytes();

        // Shape always comes first
        let shape_type = self.parse_shape_type()?;

        let shape = match shape_type {
            ShapeType::Null => self.parse_null(),
            ShapeType::Point => shape::Shape::Point(self.parse_point()?),
            ShapeType::PolyLine => self.parse_polyline()?,
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

        Ok(ShpRecord { shape })
    }
}
