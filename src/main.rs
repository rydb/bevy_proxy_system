
use std::{collections::HashSet, marker::PhantomData};

use bevy::{ecs::{schedule::ScheduleLabel, system::SystemId}, prelude::*};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScheduleRegistration::<Update>::default())
        .add_systems(Startup, add_systems)
        .run();
}

type Systems = HashSet<SystemId>;

#[derive(Resource)]
pub struct ScheduleRegistry<T: ScheduleLabel + Default>{
    systems: Systems,
    a: PhantomData<T>
}

impl<T: ScheduleLabel + Default> Default for ScheduleRegistry<T> {
    fn default() -> Self {
        Self { systems: Default::default(), a: Default::default() }
    }
}
#[derive(Default)]
pub struct ScheduleRegistration<T: ScheduleLabel + Default> {
    _a: PhantomData<T>
}

impl<T: ScheduleLabel + Default> Plugin for ScheduleRegistration<T> {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<ScheduleRegistry<T>>()
        .add_systems(T::default(), run_proxy_system::<T>);
        
    }
}

pub fn greet_user() {
    println!("hello!");
}

pub fn add_systems(
    mut commands: Commands,
    mut schedule_registry: ResMut<ScheduleRegistry<Update>>
) {
    let system_id = {
        commands.register_system(greet_user)
    };
    schedule_registry.systems.insert(system_id);
}

#[doc = "hidden"]
pub fn run_proxy_system<T: ScheduleLabel + Default>(
    schedule_registry: Res<ScheduleRegistry<T>>, 
    mut commands: Commands)
{
    for system in schedule_registry.systems.iter() {
        commands.run_system(*system);
    }
}