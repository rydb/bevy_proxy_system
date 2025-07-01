use std::{any::TypeId, collections::{HashMap, HashSet}};

use bevy::{app::DynEq, ecs::{component::ComponentId, intern::Interned, schedule::{InternedScheduleLabel, ScheduleLabel}, system::SystemId}, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bytemuck::TransparentWrapper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn greet_user() {
    println!("Hello!");
}

/// struct variant of a `ScheduleLabel` with just TypeId
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct UntypedScheduleLabel(TypeId);

impl UntypedScheduleLabel {
    pub fn new<T: ScheduleLabel>() -> Self {
        Self(TypeId::of::<T>())
    }
}

#[derive(Default, Resource)]
pub struct ScheduleRegistry {
   schedule_registry: HashMap<UntypedScheduleLabel, InternedScheduleLabel>,
   internal_schedule_registry: HashMap<InternedScheduleLabel, HashSet<SystemId>>
}

impl ScheduleRegistry {
    pub fn get<T: ScheduleLabel + Default>(&self) -> &HashSet<SystemId> {
        let untyped_label = UntypedScheduleLabel::new::<T>();

        let internal_label = self.schedule_registry.get(&untyped_label).unwrap();

        self.internal_schedule_registry.get(internal_label).unwrap()
    }
    pub fn insert<T: ScheduleLabel + Default>(&mut self, system_id: SystemId) {
        let untyped_label = UntypedScheduleLabel::new::<T>();
        let interned_label = T::default().intern();
        self.schedule_registry.insert(untyped_label, interned_label.clone());
        match self.internal_schedule_registry.get_mut(&interned_label) {
            Some(n) => {
                n.insert(system_id);
            },
            None => {
                let mut map = HashSet::new();
                map.insert(system_id);
                self.internal_schedule_registry.insert(interned_label, map);
            },
        };

    }
}

#[derive(Default)]
pub struct InternedLabelRegistry(HashMap<UntypedScheduleLabel, InternedScheduleLabel>);

pub fn test() {
    let registry = ScheduleRegistry::default();
    registry.insert::<Update>();
    let systems = registry.get::<Update>();
}

#[doc = "hidden"]
pub fn run_proxy_system<ScheduleRegistry>(
    schedule_registry: Res<ScheduleRegistry>, 
    mut commands: Commands)
{
    for (_, system) in TransparentWrapper::peel_ref(&*schedule_registry).iter() {
        commands.run_system(*system);
    }
}