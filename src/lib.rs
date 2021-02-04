//mod boxes;
//mod debug;
mod bounding_sphere;
use bevy::prelude::*;
use std::marker::PhantomData;
//use boxes::*;
//use debug::*;
pub use bounding_sphere::BoundingSphere;

//pub use debug::BoundingVolumeDebug;

pub struct BoundingVolumePlugin;
impl Plugin for BoundingVolumePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            stage::UPDATE,
            spawn_bounding_volumes::<BoundingSphere>.system(),
        )
        .add_system_to_stage(stage::POST_UPDATE, BoundingSphere::update.system());
    }
}

pub struct BoundingVolumeDebugPlugin;
impl Plugin for BoundingVolumeDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            stage::POST_UPDATE,
            spawn_debug_meshes::<BoundingSphere>.system(),
        )
        .add_system_to_stage(
            stage::POST_UPDATE,
            update_debug_mesh::<BoundingSphere>.system(),
        );
    }
}

///
#[derive(Debug, Clone)]
pub struct BoundingVolume<T: IsBoundingVolume + Send + Sync>(PhantomData<T>);
impl<T: IsBoundingVolume + Send + Sync> Default for BoundingVolume<T> {
    fn default() -> Self {
        BoundingVolume(PhantomData::default())
    }
}

pub struct BoundingVolumeDebug;

pub trait IsBoundingVolume {
    /// Initializes a valid bounding volume given a [Mesh] and [GlobalTransform].
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self;
    /// Generate a [Mesh] from the [BoundingVolume]'s own definition.
    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh;
}

/// Marks new BoundingVolumes that are awaiting their mesh to load
pub struct LoadingMesh;

/// Use generics to spawn a child entity with component type T
pub fn spawn_bounding_volumes<T: 'static + IsBoundingVolume + Send + Sync>(
    commands: &mut Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<
        (&Handle<Mesh>, &GlobalTransform, Entity),
        (Or<(Added<BoundingVolume<T>>, With<LoadingMesh>)>, Without<T>)
    >,
) {
    for (handle, transform, entity) in query.iter() {
        match meshes.get(handle) {
            Some(mesh) => {
                commands.set_current_entity(entity);
                commands.with(T::new(mesh, transform));
                commands.remove_one::<LoadingMesh>(entity);
                commands.remove_one::<BoundingVolume<T>>(entity);
                println!("mesh loaded");
            }
            None => {
                commands.set_current_entity(entity);
                commands.with(LoadingMesh);
            }
        }
    }
}

pub struct BoundingVolumeDebugMesh;

pub fn spawn_debug_meshes<T: 'static + IsBoundingVolume + Send + Sync>(
    commands: &mut Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&GlobalTransform, &T, Entity), (Added<T>, With<BoundingVolumeDebug>)>,
) {
    for (transform, bound_vol, entity) in query.iter() {
        println!("debug spawn");
        let mesh_handle = meshes.add(bound_vol.new_debug_mesh(transform));
        commands.set_current_entity(entity);
        commands.with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh_handle,
                material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                ..Default::default()
            })
            .with(BoundingVolumeDebugMesh);
        });
    }
}

pub fn update_debug_mesh<T: 'static + IsBoundingVolume + Send + Sync>(
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&T, &GlobalTransform, &Children), (Changed<T>, With<BoundingVolumeDebug>)>,
    debug_mesh_query: Query<&Handle<Mesh>, With<BoundingVolumeDebugMesh>>
) {
    for (volume, transform, children) in query.iter() {
        for child in children.iter() {
            if let Ok(mesh_handle) = debug_mesh_query.get(*child) {
                println!("update debug");
                *meshes
                    .get_mut(mesh_handle)
                    .expect("Bad handle in bounding debug mush") = volume.new_debug_mesh(transform);
            }
        }
    }
}