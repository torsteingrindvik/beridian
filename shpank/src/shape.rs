use std::ops::Range;

use crate::parse::Error;

#[derive(Debug)]
pub struct MinimumBoundingRectangle {
    pub x: Range<f64>,
    pub y: Range<f64>,
}

pub type Integer = i32;
pub type Double = f64;

#[derive(Debug)]
pub struct ShpLength(pub i32);

impl ShpLength {
    /// Convert a Shapefile length to number of bytes.
    /// Use when lengths are expressed in # of 16-bit words.
    pub fn num_bytes(&self) -> usize {
        self.0 as usize * 2
    }
}

#[derive(Debug)]
pub struct ShpFile {
    pub header: ShpHeader,
    pub records: Vec<ShpRecord>,
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

impl ShpHeader {
    pub const FILE_CODE: i32 = 0x0000270a;
}

#[derive(Debug)]
pub struct ShpRecordHeader {
    /// Starting at 1
    pub record_number: i32,

    /// In 16-bit words.
    /// This is not including the record header.
    pub content_length: ShpLength,
}

#[derive(Debug)]
pub struct ShpRecord {
    pub shape: Shape,
}

#[derive(Debug)]
pub enum ShapeType {
    Null = 0,
    Point = 1,
    PolyLine = 3,
    Polygon = 5,
    MultiPoint = 8,
    PointZ = 11,
    PolylineZ = 13,
    PolygonZ = 15,
    MultiPointZ = 18,
    PointM = 21,
    PolylineM = 23,
    PolygonM = 25,
    MultiPointM = 28,
    MultiPatch = 31,
}

impl TryFrom<i32> for ShapeType {
    type Error = Error;

    fn try_from(value: i32) -> crate::parse::Result<Self> {
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
pub enum Shape {
    Null,
    Point(Point),
    PolyLine(PolyLine),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    PointZ(PointZ),
    PolylineZ(PolylineZ),
    PolygonZ(PolygonZ),
    MultiPointZ(MultiPointZ),
    PointM(PointM),
    PolylineM(PolylineM),
    PolygonM(PolygonM),
    MultiPointM(MultiPointM),
    MultiPatch(MultiPatch),
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct PolyLine {
    pub mbr: MinimumBoundingRectangle,
    pub parts: Vec<i32>,
    pub points: Vec<Point>,
}

#[derive(Debug)]
pub struct Polygon {
    pub mbr: MinimumBoundingRectangle,
    pub parts: Vec<i32>,
    pub points: Vec<Point>,
}

#[derive(Debug)]
pub struct MultiPoint;

#[derive(Debug)]
pub struct PointZ;

#[derive(Debug)]
pub struct PolylineZ;

#[derive(Debug)]
pub struct PolygonZ;

#[derive(Debug)]
pub struct MultiPointZ;

#[derive(Debug)]
pub struct PointM;

#[derive(Debug)]
pub struct PolylineM;

#[derive(Debug)]
pub struct PolygonM;

#[derive(Debug)]
pub struct MultiPointM;

#[derive(Debug)]
pub struct MultiPatch;
