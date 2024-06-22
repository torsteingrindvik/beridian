use bevy::math::DVec2;
use serde::{Deserialize, Serialize};
use shpank::shape::Shape;

use crate::ecs_geo::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub name: Option<GeoName>,
    pub feature: GeoFeature,
    pub variant: Variant,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Variant {
    Point(GeoPoint),
    Line(GeoLines),
    Polygon(GeoPolygon),
}

impl From<shpank::spatial::Object> for Object {
    fn from(object: shpank::spatial::Object) -> Self {
        Self {
            name: if object.name.is_empty() {
                None
            } else {
                Some(GeoName(object.name))
            },
            feature: GeoFeature(object.fclass),
            variant: match object.shape {
                Shape::Point(point) => Variant::Point(GeoPoint(DVec2::new(point.x, point.y))),
                Shape::PolyLine(polyline) => {
                    if polyline.parts.len() > 1 {
                        unimplemented!()
                    }

                    Variant::Line({
                        let points = polyline
                            .points
                            .into_iter()
                            .map(|p| DVec2::new(p.x, p.y))
                            .collect();
                        GeoLines { line: points }
                    })
                }
                Shape::Polygon(polygon) => {
                    let total_points = polygon.points.len();
                    let mut points_added = 0;
                    let mut results = vec![];

                    for window in polygon.parts.windows(2) {
                        let [start, stop] = TryInto::<[_; 2]>::try_into(window).unwrap();
                        let [start, stop] = [start as usize, stop as usize];

                        let part: Vec<DVec2> = polygon.points[start..stop]
                            .iter()
                            .copied()
                            .map(|p| DVec2::new(p.x, p.y))
                            .collect();
                        points_added += part.len();
                        results.push(part);
                    }

                    let last = *polygon.parts.last().unwrap() as usize;

                    let part: Vec<DVec2> = polygon.points[last..]
                        .iter()
                        .copied()
                        .map(|p| DVec2::new(p.x, p.y))
                        .collect();
                    points_added += part.len();
                    results.push(part);

                    assert_eq!(total_points, points_added);
                    Variant::Polygon(GeoPolygon { polygon: results })
                }
                others => unimplemented!("missing impl: {others:?}"),
            },
        }
    }
}
