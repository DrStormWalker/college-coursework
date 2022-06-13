use anyhow::Result as AnyResult;
use specs::{DispatcherBuilder, World, WorldExt};
use tokio::time;

use crate::simulation::{
    DeltaTime, Identifier, Mass, Position, Printer, Simulator, Velocity, PLANET_EARTH,
    PLANET_JUPITER, PLANET_MARS, PLANET_MERCURY, PLANET_NEPTUNE, PLANET_SATURN, PLANET_URANUS,
    PLANET_VENUS, SUN,
};

pub fn run() -> AnyResult<()> {
    // Create the 'world', a container for the components and other
    // resources used by the entities
    let mut world = World::new();
    world.register::<Identifier>();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();

    SUN.register_entity(&mut world);
    PLANET_MERCURY.register_entity(&mut world);
    PLANET_VENUS.register_entity(&mut world);
    PLANET_EARTH.register_entity(&mut world);
    PLANET_MARS.register_entity(&mut world);
    PLANET_JUPITER.register_entity(&mut world);
    PLANET_SATURN.register_entity(&mut world);
    PLANET_URANUS.register_entity(&mut world);
    PLANET_NEPTUNE.register_entity(&mut world);

    world.insert(DeltaTime(86400.0));

    let mut simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .build();

    let mut print_dispatcher = DispatcherBuilder::new()
        .with(Printer::new(), "sys_printer", &[])
        .build();

    print_dispatcher.dispatch(&mut world);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        const FRAME_RATE: f64 = 60.0;
        const NUM_ITERATIONS_PER_FRAME: usize = 60;

        const SIMULATION_SECONDS_PER_SECOND: f64 = (60 * 60 * 24) as f64;

        let mut frame_interval = time::interval(time::Duration::from_secs_f64(1.0 / FRAME_RATE));
        let mut start_time = time::Instant::now();

        let mut frame_count = 0;

        while true {
            frame_interval.tick().await;

            let now = time::Instant::now();
            let dt = (now - start_time).as_secs_f64();
            start_time = now;

            {
                let delta_time: &mut DeltaTime = world.get_mut().unwrap();

                delta_time.0 = dt * SIMULATION_SECONDS_PER_SECOND / NUM_ITERATIONS_PER_FRAME as f64;
            }

            (0..NUM_ITERATIONS_PER_FRAME)
                .into_iter()
                .for_each(|_| simulation_dispatcher.dispatch(&mut world));

            frame_count += 1;

            if frame_count % FRAME_RATE as u64 == 0 {
                print_dispatcher.dispatch(&world);
            }
        }
    });

    print_dispatcher.dispatch(&mut world);

    // Program ran successfully
    Ok(())
}
