/*use anyhow::Result as AnyResult;
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

    // Register the Sun and all the planets as entities
    SUN.register_entity(&mut world);
    PLANET_MERCURY.register_entity(&mut world);
    PLANET_VENUS.register_entity(&mut world);
    PLANET_EARTH.register_entity(&mut world);
    PLANET_MARS.register_entity(&mut world);
    PLANET_JUPITER.register_entity(&mut world);
    PLANET_SATURN.register_entity(&mut world);
    PLANET_URANUS.register_entity(&mut world);
    PLANET_NEPTUNE.register_entity(&mut world);

    //Initialise the dispatchers for the ECS Systsmes
    let mut simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .build();

    let mut print_dispatcher = DispatcherBuilder::new()
        .with(Printer::new(), "sys_printer", &[])
        .build();

    // Call the print dispatcher
    print_dispatcher.dispatch(&mut world);

    // Setup a new async runtime throwing an error if it did not
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Start the aysnc runtime blocking the current thread
    runtime.block_on(async {
        // The number of frames per second
        const FRAME_RATE: f64 = 60.0;
        // The number of iterations of the simulation to do per frame
        // The higher the number the more accurate the simulation
        const NUM_ITERATIONS_PER_FRAME: usize = 60;

        const SIMULATION_SECONDS_PER_SECOND: f64 = (60 * 60 * 24) as f64;

        // Add the delta time into the ECS world
        world.insert(DeltaTime(SIMULATION_SECONDS_PER_SECOND));

        // Initialise the intervals at which to run a new frame
        let mut frame_interval = time::interval(time::Duration::from_secs_f64(1.0 / FRAME_RATE));
        let mut start_time = time::Instant::now();

        let mut frame_count = 0;

        // Start the program loop
        loop {
            // Wait until a new frame is needed
            frame_interval.tick().await;

            // Calculate the time since the last frame was used
            let now = time::Instant::now();
            let dt = (now - start_time).as_secs_f64();
            start_time = now;

            // Update the delta time resource within the ECS world
            {
                let delta_time: &mut DeltaTime = world.get_mut().unwrap();

                delta_time.0 = dt * SIMULATION_SECONDS_PER_SECOND / NUM_ITERATIONS_PER_FRAME as f64;
            }

            // Run the simulation the number of times specified
            (0..NUM_ITERATIONS_PER_FRAME)
                .into_iter()
                .for_each(|_| simulation_dispatcher.dispatch(&mut world));

            // Increment the fram count
            frame_count += 1;

            // If FRAME_COUNT number of frames have passed then print the information
            // about each body
            if frame_count % FRAME_RATE as u64 == 0 {
                print_dispatcher.dispatch(&world);
            }
        }
    });

    // At the end of the program print the final locations of the bodies
    print_dispatcher.dispatch(&mut world);

    // Exit telling the calling scope that it ran successfully
    Ok(())
}*/
