//! ECS versions of geodata

use bevy::{math::DVec2, prelude::*};
use serde::{Deserialize, Serialize};
use shpank::spatial::Fclass;

#[derive(Debug, Component, Deref, Serialize, Deserialize, Clone)]
pub struct GeoName(pub String);

#[derive(Debug, Component, Deref, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeoFeature(pub Fclass);

#[derive(Debug, Component, Deref, Serialize, Deserialize, Clone, Copy)]
pub struct GeoPoint(pub DVec2);

#[derive(Debug, Component, Deref, Serialize, Deserialize, Clone)]
pub struct GeoLines {
    // TODO: Likely actually vec of vec
    pub line: Vec<DVec2>,
}
#[derive(Debug, Component, Deref, Serialize, Deserialize, Clone)]
pub struct GeoPolygon {
    pub polygon: Vec<Vec<DVec2>>,
}
