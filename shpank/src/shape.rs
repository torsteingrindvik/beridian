use std::mem::size_of;

use crate::parse::{Error::UnexpectedData, Parser, Result};

#[derive(Debug)]
pub enum ShapeType {
    Null = 0,
    Point = 1,
    PolyLine = 3,

    // wiki: MBR, Number of parts, Number of points, Parts, Points
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

impl Shape {
    pub fn parse(type_: &ShapeType, data: &[u8]) -> Result<Self> {
        match type_ {
            ShapeType::Null => Self::parse_null(data),
            ShapeType::Point => unimplemented!(),
            ShapeType::PolyLine => unimplemented!(),
            ShapeType::Polygon => Self::parse_polygon(data),
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
        }
    }

    pub fn parse_null(data: &[u8]) -> Result<Self> {
        if data.len() != 0 {
            Err(UnexpectedData(format!(
                "Expected empty data buffer for the Null type"
            )))
        } else {
            Ok(Self::Null)
        }
    }

    pub fn parse_polygon(data: &[u8]) -> Result<Self> {
        let mut parser = Parser::with_reader(data);

        let mbr = parser.consume_and_parse_mbr()?;

        let num_parts = parser.consume_and_parse_integer()?;
        let num_points = parser.consume_and_parse_integer()?;

        dbg!(
            "Polygon: ",
            &data.len(),
            &mbr,
            &num_parts,
            &num_points,
            parser.num_bytes_read()
        );

        let mut parts_array = Vec::with_capacity(4 * num_parts as usize);

        for _ in 0..num_parts {
            let part_idx = parser.consume_and_parse_integer()?;
            dbg!(&part_idx);
            parts_array.push(part_idx);
        }

        let mut points_array = Vec::with_capacity(size_of::<Point>() * num_points as usize);
        for _ in 0..num_points {
            let point = parser.consume_and_parse_point()?;
            dbg!(&point);
            points_array.push(point);
        }

        debug_assert_eq!(data.len(), parser.num_bytes_read());

        // 164 bytes total, 40 consumed -> 124 left
        // NumParts[1] -> should read an i32, so 120 left
        // Point[8] -> should read 8 (2xDouble) -> 8 *16

        unimplemented!()
    }
}

#[derive(Debug)]
pub struct PolyLine;

#[derive(Debug)]
pub struct Polygon {
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
