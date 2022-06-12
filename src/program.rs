use anyhow::Result as AnyResult;
use specs::{Builder, DispatcherBuilder, World, WorldExt};

use crate::simulation::{Mass, Position, Simulator, Velocity, SUN};

pub fn run() -> AnyResult<()> {
    // Create the 'world', a container for the components and other
    // resources used by the entities
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();

    world
        .create_entity()
        .with(SUN.get_pos())
        .with(SUN.get_vel())
        .with(SUN.get_mass())
        .build();
    world.create_entity().with().build();

    let mut simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .build();

    simulation_dispatcher.dispatch(&mut world);
    simulation_dispatcher.dispatch(&mut world);

    // Program ran successfully
    Ok(())
}
