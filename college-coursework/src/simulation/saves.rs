use std::{
    fs,
    io::{Read as _, Write as _},
    sync::{mpsc, Arc},
};

use cgmath::{Quaternion, Vector3, Zero};
use chrono::Utc;
use dialog::DialogBox;
use serde::{Deserialize, Serialize};
use specs::{
    Builder, Entities, Join, Read, ReadExpect, ReadStorage, World, WorldExt, Write, WriteStorage,
};

use crate::{
    models::sphere::Icosphere,
    panel::PlanetWindowShown,
    renderer::{
        camera::{CameraPosition, CameraSpeed},
        components::{PlanetColour, RenderModel},
        instance::Instance,
    },
};

use super::{
    BodyType, GravitationalConstant, Identifier, InteractionFlags, InteractionHandler, Mass,
    Position, TimeScale, Velocity,
};

#[derive(Serialize, Deserialize)]
pub struct TimeState {
    date_time: String,
    time_scale: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ConstantState {
    gravitational_constant: f64,
}

#[derive(Serialize, Deserialize)]
pub struct CameraState {
    #[serde(rename = "position")]
    camera_position: [f32; 3],
    #[serde(rename = "speed")]
    camera_speed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct PlanetState {
    id: String,
    name: String,
    position: [f64; 3],
    velocity: [f64; 3],
    mass: f64,
    colour: [f32; 4],
}

pub type PlanetsState = Vec<PlanetState>;

#[derive(Serialize, Deserialize)]
pub struct SimulationState {
    #[serde(rename = "time")]
    time_state: TimeState,
    #[serde(rename = "constants")]
    constant_state: ConstantState,
    #[serde(rename = "camera")]
    camera_state: CameraState,

    #[serde(rename = "planet")]
    planet_state: PlanetsState,
}
impl SimulationState {
    pub fn serialize_from_world(world: &mut World) -> Self {
        world.exec(
            |(
                camera_position,
                camera_speed,
                gravitational_constant,
                time_scale,
                planet_ids,
                planet_colours,
                planet_positions,
                planet_velocities,
                planet_masses,
            ): (
                Read<CameraPosition>,
                Read<CameraSpeed>,
                Read<GravitationalConstant>,
                Read<TimeScale>,
                ReadStorage<Identifier>,
                ReadStorage<PlanetColour>,
                ReadStorage<Position>,
                ReadStorage<Velocity>,
                ReadStorage<Mass>,
            )| {
                let planet_state = (
                    &planet_ids,
                    &planet_colours,
                    &planet_positions,
                    &planet_velocities,
                    &planet_masses,
                )
                    .join()
                    .map(|(id, colour, position, velocity, mass)| PlanetState {
                        id: id.get_id().to_string(),
                        name: id.get_name().to_string(),
                        position: position.0.into(),
                        velocity: velocity.0.into(),
                        mass: mass.0,
                        colour: colour.0,
                    })
                    .collect();

                Self {
                    time_state: TimeState {
                        date_time: Utc::now().to_rfc3339(),
                        time_scale: time_scale.total_time_elapsed,
                    },
                    constant_state: ConstantState {
                        gravitational_constant: gravitational_constant.0,
                    },
                    camera_state: CameraState {
                        camera_position: camera_position.0.into(),
                        camera_speed: camera_speed.0,
                    },
                    planet_state,
                }
            },
        )
    }

    pub fn deserialize_to_world(self, world: &mut World) {
        world.exec(
            |(
                mut camera_position,
                mut camera_speed,
                mut gravitational_constant,
                mut time_scale,
                planet_ids,
                planet_colours,
                planet_positions,
                planet_velocities,
                planet_masses,
                entities,
            ): (
                Write<CameraPosition>,
                Write<CameraSpeed>,
                Write<GravitationalConstant>,
                Write<TimeScale>,
                WriteStorage<Identifier>,
                WriteStorage<PlanetColour>,
                WriteStorage<Position>,
                WriteStorage<Velocity>,
                WriteStorage<Mass>,
                Entities,
            )| {
                camera_position.0 = self.camera_state.camera_position.into();
                camera_speed.0 = self.camera_state.camera_speed.into();

                gravitational_constant.0 = self.constant_state.gravitational_constant;

                *time_scale =
                    TimeScale::from_max_time_per_iteration(self.time_state.time_scale, 86400.0);

                (
                    &planet_ids,
                    &planet_positions,
                    &planet_colours,
                    &planet_velocities,
                    &planet_masses,
                    &entities,
                )
                    .join()
                    .filter(|(id, _col, _pos, _vel, _mass, _entity)| id.get_id() != "sun")
                    .for_each(|(_id, _col, _pos, _vel, _mass, entity)| {
                        entities.delete(entity).unwrap();
                    });
            },
        );

        let (device, queue, texture_bind_group_layout) = {
            let device = (*world.fetch::<Arc<wgpu::Device>>()).clone();
            let queue = (*world.fetch::<Arc<wgpu::Queue>>()).clone();
            let texture_bind_group_layout = (*world.fetch::<Arc<wgpu::BindGroupLayout>>()).clone();

            (device, queue, texture_bind_group_layout)
        };

        self.planet_state
            .into_iter()
            .filter(|state| state.id != "sun")
            .for_each(|state| {
                world
                    .create_entity()
                    .with(Identifier::new(state.id.clone(), state.name))
                    .with(PlanetWindowShown::default())
                    .with(Position(state.position.into()))
                    .with(Velocity(state.velocity.into()))
                    .with(Mass(state.mass))
                    .with(PlanetColour(state.colour))
                    .with(RenderModel::new(
                        &device,
                        Icosphere::new(2.5, 3).into_model(
                            &device,
                            &queue,
                            state.id.clone(),
                            state.colour,
                            &texture_bind_group_layout,
                        ),
                        Instance::new(
                            Vector3::from(state.position).map(|a| a as f32) / 4_000_000_000.0,
                            Quaternion::zero(),
                        ),
                        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        Some(&state.id),
                    ))
                    .with(InteractionHandler::new(
                        InteractionFlags::all(),
                        BodyType::Planet,
                    ))
                    .build();
            });

        world.maintain();
    }

    fn save_string(contents: String) {
        std::thread::spawn(move || {
            let file_location = dialog::FileSelection::new("Save Simulation")
                .title("Save Simulation")
                .mode(dialog::FileSelectionMode::Save)
                .show()
                .expect("Could not display dialog box");

            if let Some(file_location) = file_location {
                let file = fs::File::options()
                    .write(true)
                    .create(true)
                    .open(file_location);

                match file {
                    Ok(mut file) => match file.write_all(contents.as_bytes()) {
                        Err(err) => dialog::Message::new(format!("{:?}", err))
                            .title("Failed to write to file.")
                            .show()
                            .expect("Could not display dialog box"),
                        _ => {}
                    },
                    Err(err) => dialog::Message::new(format!("{:?}", err))
                        .title("Failed to save file.")
                        .show()
                        .expect("Could not display dialog box"),
                }
            }
        });
    }

    pub fn save_json(self) -> Result<(), serde_json::Error> {
        let contents = serde_json::to_string_pretty(&self)?;

        Self::save_string(contents);

        Ok(())
    }

    pub fn save_toml(self) -> Result<(), toml::ser::Error> {
        let contents = toml::to_string_pretty(&self)?;

        Self::save_string(contents);

        Ok(())
    }
}

pub struct SaveHandler {
    load_receiver: mpsc::Receiver<SimulationState>,
    load_sender: mpsc::Sender<SimulationState>,
}
impl SaveHandler {
    pub fn new() -> Self {
        let (load_sender, load_receiver) = mpsc::channel();

        Self {
            load_sender,
            load_receiver,
        }
    }

    fn load_string() -> Option<String> {
        let file_location = dialog::FileSelection::new("Load Simulation")
            .title("Load Simulation")
            .mode(dialog::FileSelectionMode::Open)
            .show()
            .expect("Could not display dialog box");

        if let Some(file_location) = file_location {
            let file = fs::File::open(file_location);

            match file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Err(err) => dialog::Message::new(format!("{:?}", err))
                            .title("Failed to load file.")
                            .show()
                            .expect("Could not display dialog box"),
                        Ok(len) => return Some(contents[..len].to_string()),
                    }
                }
                Err(err) => dialog::Message::new(format!("{:?}", err))
                    .title("Failed to load file.")
                    .show()
                    .expect("Could not display dialog box"),
            }
        }

        None
    }

    pub fn load_toml(&self) {
        let sender = self.load_sender.clone();
        std::thread::spawn(move || {
            let contents = Self::load_string();

            if let Some(contents) = contents {
                let state = toml::from_str::<SimulationState>(&contents);

                match state {
                    Ok(state) => sender.send(state).unwrap(),
                    Err(err) => dialog::Message::new(format!("{:?}", err))
                        .title("Invalid file format.")
                        .show()
                        .expect("Could not display dialog box"),
                }
            }
        });
    }

    pub fn load_json(&self) {
        let sender = self.load_sender.clone();
        std::thread::spawn(move || {
            let contents = Self::load_string();

            if let Some(contents) = contents {
                let state = serde_json::from_str::<SimulationState>(&contents);

                match state {
                    Ok(state) => sender.send(state).unwrap(),
                    Err(err) => dialog::Message::new(format!("{:?}", err))
                        .title("Invalid file format.")
                        .show()
                        .expect("Could not display dialog box"),
                }
            }
        });
    }

    pub fn try_load_state(&mut self) -> Result<SimulationState, mpsc::TryRecvError> {
        self.load_receiver.try_recv()
    }
}
