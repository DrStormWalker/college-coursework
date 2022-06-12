use anyhow::Result as AnyResult;
use specs::{DispatcherBuilder, World, WorldExt};

use crate::simulation::{
    DeltaTime, Mass, Position, Printer, Simulator, Velocity, PLANET_EARTH, PLANET_MARS,
    PLANET_MERCURY, PLANET_VENUS, SUN,
};

pub fn run() -> AnyResult<()> {
    // Create the 'world', a container for the components and other
    // resources used by the entities
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();

    SUN.register_entity(&mut world);
    PLANET_MERCURY.register_entity(&mut world);
    PLANET_VENUS.register_entity(&mut world);
    PLANET_EARTH.register_entity(&mut world);
    PLANET_MARS.register_entity(&mut world);

    world.insert(DeltaTime(86400.0));

    let mut simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .build();

    let mut print_dispatcher = DispatcherBuilder::new()
        .with(Printer::new(), "sys_printer", &[])
        .build();

    print_dispatcher.dispatch(&mut world);

    simulation_dispatcher.dispatch(&mut world);
    simulation_dispatcher.dispatch(&mut world);

    print_dispatcher.dispatch(&mut world);

    // Program ran successfully
    Ok(())
}
