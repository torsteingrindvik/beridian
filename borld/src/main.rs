use argh::FromArgs;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use borld::{
    ecs_geo::GeoFeature,
    feature_data::feature_scale,
    preprocess::{Object, Variant},
};
use std::path::PathBuf;

#[derive(Debug, FromArgs)]
/// read a processed .borld file and show
struct Args {
    /// processed .borld files containing object data
    #[argh(positional)]
    borlds: Vec<PathBuf>,
}

#[derive(Debug, Component, Deref)]
struct ObjectSourceFileIndex(usize);

#[derive(Debug, Resource, Deref)]
struct SourceFiles {
    mapping: HashMap<usize, PathBuf>,
}

#[derive(Debug, Resource, Clone)]
struct LoadObjectsPlugin {
    paths: Vec<PathBuf>,
}

impl Plugin for LoadObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .init_resource::<FeatureCache>()
            .add_systems(Startup, load_all)
            .add_systems(Update, spawn_new);
    }
}

fn load_all(mut commands: Commands, plugin: Res<LoadObjectsPlugin>) {
    let mut source_files = HashMap::default();

    let mut to_load = vec![];
    for borld in plugin.paths.iter() {
        let bytes = std::fs::read(borld).unwrap();
        let objects: Vec<Object> = bincode::deserialize(&bytes).unwrap();

        to_load.push((objects, borld.clone()));
    }

    info!("Loading into ECS");
    for (index, (object_collection, source_file)) in to_load.into_iter().enumerate() {
        source_files.insert(index, source_file);

        for Object {
            name,
            feature,
            variant,
        } in object_collection
        {
            let mut cmds = commands.spawn((feature, ObjectSourceFileIndex(index)));

            match variant {
                Variant::Point(p) => cmds.insert(p),
                Variant::Line(l) => cmds.insert(l),
                Variant::Polygon(p) => cmds.insert(p),
            };

            if let Some(name) = name {
                cmds.insert(name);
            }
        }
    }

    commands.insert_resource(SourceFiles {
        mapping: source_files,
    });

    info!("Done");
}

#[derive(Debug)]
struct FeatureData {
    scale: Vec3,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

#[derive(Debug, Default, Deref, DerefMut, Resource)]
struct FeatureCache {
    scales: HashMap<GeoFeature, FeatureData>,
}

fn spawn_new(
    mut commands: Commands,
    unmeshed: Query<(Entity, &GeoFeature), Without<Transform>>,
    mut cache: ResMut<FeatureCache>,
) {
    for (entity, feature) in &unmeshed {
        let data = cache.entry(*feature).or_insert_with(|| FeatureData {
            scale: feature_scale(feature),
            material: todo!(),
            mesh: todo!(),
        });
    }
}

fn main() {
    let Args { borlds } = argh::from_env();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(LoadObjectsPlugin { paths: borlds });

    app.run();
}
