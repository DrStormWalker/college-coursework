mod args;
mod assets;
mod log;
mod models;
mod panel;
mod renderer;
mod setup;
mod simulation;
mod test;
mod util;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

use std::{error::Error, fmt, thread};

use ::log::info;
use anyhow::Result as AnyResult;
use crossbeam::channel;
use error_stack::{IntoReport, Result, ResultExt};
use setup::SetupError;
use specs::{Join, ReadStorage};
use thiserror::Error;
use tokio::io;

use crate::{args::Args, simulation::Identifier};
use clap::Parser;

const APPLICATION_NAME: &'static str = crate_name!();
const APPLICATION_AUTHOR: &'static str = crate_authors!();
const APPLICATION_VERSION: &'static str = crate_version!();

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to setup application")]
    SetupError,

    #[error("Failed to build Async Runtime")]
    RuntimeBuildError,
}

fn main() -> Result<(), ApplicationError> {
    let args = Args::parse();

    log::setup_log().unwrap();

    // Logs use the 'trace', 'debug', 'info', 'warn' and 'error' macros.
    // Corresponding to their repective log levels
    info!("--------------------------------");
    info!("--       Program started      --");
    info!("--------------------------------");
    info!("Logging initialised");

    // Declare if running in debug mode
    #[cfg(debug_assertions)]
    info!("Running in debug mode");

    // Setup a new async runtime throwing an error if it did not
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .report()
        .attach_printable("Failed to build Async Runtime")
        .change_context(ApplicationError::RuntimeBuildError)?;

    // Run the setup code within an async runtime
    let (window, world, dispatchers) = runtime
        .block_on(async {
            // Create the main window
            let window = crate::renderer::window::Window::new().await;

            // Setup the Entity Component System
            let (world, dispatchers) = setup::setup(
                &window.state.device,
                window.state.queue.clone(),
                &window.state.texture_bind_group_layout,
            )
            .await
            .attach_printable("Failed to set up application")?;

            Ok((window, world, dispatchers))
        })
        .change_context(ApplicationError::SetupError)?;

    // Get all the identifiers of the registered planets
    let ids = {
        let (ids,): (ReadStorage<Identifier>,) = world.system_data();

        (&ids).join().map(|id| id.clone()).collect::<Vec<_>>()
    };

    // Run the main loop
    window.run(world, dispatchers);
    //Ok(())
}
