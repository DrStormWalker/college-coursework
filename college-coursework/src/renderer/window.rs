use std::{
    fs::File,
    io::{BufReader, Cursor},
    thread,
};

use log::error;
use rodio::{decoder::DecoderError, Decoder, OutputStream, Sink};
use specs::World;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{renderer::state::State, setup::Dispatchers};

/// Data structure representing the program window
pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
    pub state: State,
}
impl Window {
    pub async fn new() -> Self {
        //! Create a new window
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        // Initialise the program state
        let state = State::new(&window, &event_loop).await;

        Self {
            event_loop,
            window,
            state,
        }
    }

    pub fn run(self, mut world: World, mut dispatchers: Dispatchers<'static, 'static>) -> ! {
        //! Runs the program

        let Self {
            event_loop,
            window,
            mut state,
        } = self;

        // Register music
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        const CITY_OF_GHOSTS_MUSIC: &[u8] =
            include_bytes!("../../assets/music/background/City of Ghosts.mp3");
        const DUST_TO_DUST_MUSIC: &[u8] =
            include_bytes!("../../assets/music/background/Dust to Dust.mp3");
        const NORTHWARD_MUSIC: &[u8] =
            include_bytes!("../../assets/music/background/Northward.mp3");
        const SLEEPING_LIGHTLY_MUSIC: &[u8] =
            include_bytes!("../../assets/music/background/Sleeping Lightly.mp3");
        const STRATUS_MUSIC: &[u8] = include_bytes!("../../assets/music/background/Stratus.mp3");

        // Spawn a thread to play music
        thread::spawn(move || {
            let files = [
                CITY_OF_GHOSTS_MUSIC,
                DUST_TO_DUST_MUSIC,
                NORTHWARD_MUSIC,
                SLEEPING_LIGHTLY_MUSIC,
                STRATUS_MUSIC,
            ];

            // Create a new music sink
            let sink = Sink::try_new(&stream_handle).unwrap();

            let mut song_num = 0;

            loop {
                // Decode a file, the file is picked from the list of songs and it will
                // repeat after there are no new songs to play
                let file = BufReader::new(Cursor::new(files[song_num % files.len()]));
                let source = Decoder::new(file).unwrap();

                // Play the file
                sink.append(source);

                // Wait until the file has finished playing
                sink.sleep_until_end();
                song_num += 1;
            }
        });

        // Create the start time for delta time
        let mut last_render_time = instant::Instant::now();

        use winit::{event::*, event_loop::ControlFlow};

        // Start the event loop
        event_loop.run(move |event, _, control_flow| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => state
                .camera_controller
                .process_mouse_move_event(delta.0, delta.1),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.on_event(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: keyboard_state,
                                    virtual_keycode: Some(virtual_keycode),
                                    ..
                                },
                            ..
                        } => state
                            .camera_controller
                            .process_keyboard_event(*virtual_keycode, *keyboard_state),
                        WindowEvent::MouseInput {
                            state: keyboard_state,
                            button,
                            ..
                        } => state
                            .camera_controller
                            .process_mouse_button_event(*button, *keyboard_state),
                        WindowEvent::MouseWheel { delta, .. } => {
                            state.camera_controller.process_mouse_scroll_event(*delta)
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Calculate delta time
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                // Update the program state using delta time
                state.update(dt, &mut world, &mut dispatchers);

                // Render the next frame
                match state.render(&mut world, &window) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => error!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        })
    }
}
