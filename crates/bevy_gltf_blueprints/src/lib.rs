pub mod spawn_from_blueprints;
pub use spawn_from_blueprints::*;

pub mod spawn_post_process;
pub(crate) use spawn_post_process::*;

pub mod animation;
pub use animation::*;

pub mod aabb;
pub use aabb::*;

pub mod clone_entity;
pub use clone_entity::*;

use core::fmt;
use std::path::PathBuf;

use bevy::{prelude::*, render::primitives::Aabb, utils::HashMap};
use bevy_gltf_components::{ComponentsFromGltfPlugin, GltfComponentsSet};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// set for the two stages of blueprint based spawning :
pub enum GltfBlueprintsSet {
    Spawn,
    AfterSpawn,
}

#[derive(Bundle)]
pub struct BluePrintBundle {
    pub blueprint: BlueprintName,
    pub spawn_here: SpawnHere,
    pub transform: TransformBundle,
}
impl Default for BluePrintBundle {
    fn default() -> Self {
        BluePrintBundle {
            blueprint: BlueprintName("default".into()),
            spawn_here: SpawnHere,
            transform: TransformBundle::default(),
        }
    }
}

#[derive(Clone, Resource)]
pub struct BluePrintsConfig {
    pub(crate) format: GltfFormat,
    pub(crate) library: BlueprintsLibrary,
    pub(crate) aabbs: bool,

    pub(crate) aabb_cache: HashMap<String, Aabb>, // cache for aabbs
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GltfFormat {
    #[default]
    GLB,
    GLTF,
}

impl fmt::Display for GltfFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GltfFormat::GLB => {
                write!(f, "glb",)
            }
            GltfFormat::GLTF => {
                write!(f, "gltf")
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Plugin for gltf blueprints
pub struct BlueprintsPlugin {
    pub format: GltfFormat,
    /// The base folder or files where library/blueprints assets are loaded from, relative to the executable.
    pub library: BlueprintsLibrary,
    pub aabbs: bool,
}

/// Specifies whether to load a `Folder` or list of `Files`. Note that in WASM
/// environments, using a `Folder` will result in an error.
#[derive(Debug, Clone)]
pub enum BlueprintsLibrary {
    Folder(PathBuf),
    Files(Vec<PathBuf>),
}

impl Default for BlueprintsPlugin {
    fn default() -> Self {
        Self {
            format: GltfFormat::GLB,
            library: BlueprintsLibrary::Folder(PathBuf::from("models/library")),
            aabbs: false,
        }
    }
}

fn aabbs_enabled(blueprints_config: Res<BluePrintsConfig>) -> bool {
    blueprints_config.aabbs
}

impl Plugin for BlueprintsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentsFromGltfPlugin)
            .register_type::<BlueprintName>()
            .register_type::<SpawnHere>()
            .register_type::<Animations>()
            .insert_resource(BluePrintsConfig {
                format: self.format.clone(),
                library: self.library.clone(),
                aabbs: self.aabbs,
                aabb_cache: HashMap::new(),
            })
            .configure_sets(
                Update,
                (GltfBlueprintsSet::Spawn, GltfBlueprintsSet::AfterSpawn)
                    .chain()
                    .after(GltfComponentsSet::Injection),
            )
            .add_systems(
                Update,
                (
                    spawn_from_blueprints,
                    compute_scene_aabbs.run_if(aabbs_enabled),
                    apply_deferred.run_if(aabbs_enabled),
                )
                    .chain()
                    .in_set(GltfBlueprintsSet::Spawn),
            )
            .add_systems(
                Update,
                (
                    update_spawned_root_first_child,
                    apply_deferred,
                    cleanup_scene_instances,
                    apply_deferred,
                )
                    .chain()
                    .in_set(GltfBlueprintsSet::AfterSpawn),
            );
    }
}
