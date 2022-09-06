use log::error;
use specs::World;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{renderer::state::State, setup::Dispatchers};

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
    pub state: State,
}
impl Window {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let state = State::new(&window).await;

        Self {
            event_loop,
            window,
            state,
        }
    }

    pub async fn run(self, mut world: World, mut dispatchers: Dispatchers<'static, 'static>) -> ! {
        let Self {
            event_loop,
            window,
            mut state,
        } = self;

        let mut last_render_time = instant::Instant::now();

        use winit::{event::*, event_loop::ControlFlow};

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
                if !state.input(event) {
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
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt, &mut world, &mut dispatchers);
                match state.render(&mut world) {
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
