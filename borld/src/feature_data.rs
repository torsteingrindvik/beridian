use bevy::math::Vec3;

use crate::ecs_geo::GeoFeature;

pub fn feature_scale(feature: &GeoFeature) -> Vec3 {
    use shpank::spatial::Fclass as F;

    match feature.0 {
        F::Airport => Vec3::default(),
        others => unimplemented!("no scale set for feature {others:?}"),
    }
}
