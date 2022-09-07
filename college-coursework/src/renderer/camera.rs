use std::f32::consts::FRAC_PI_2;

use cgmath::{
    perspective, Angle, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4, Point3, Quaternion,
    Rad, Rotation, Transform, Vector3,
};
use instant::Duration;
use specs::{Component, VecStorage};
use winit::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
};

use crate::renderer::camera;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

/*pub struct Camera {
    pub(crate) eye: cgmath::Point3<f32>,
    pub(crate) target: cgmath::Point3<f32>,
    pub(crate) up: cgmath::Vector3<f32>,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

impl Camera {
    fn build_view_projecttion_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }arg}*/

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct CameraPosition(pub Point3<f32>);
impl Default for CameraPosition {
    fn default() -> Self {
        Self(Point3::new(0.0, 0.0, 0.0))
    }
}

#[derive(Debug, Component, Default)]
#[storage(VecStorage)]
pub struct CameraSpeed(pub f32);

/*pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }

        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }

        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}*/

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    rotation: Quaternion<f32>,
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>, R: Into<Quaternion<f32>>>(position: P, rotation: R) -> Self {
        Self {
            position: position.into(),
            rotation: rotation.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            self.position,
            self.position + self.rotation.rotate_vector(Vector3::unit_z()),
            self.rotation.rotate_vector(Vector3::unit_y()),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

/*#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>, R: Into<Quaternion<f32>>>(position: P, rotation: R) -> Self {
        Self {
            position: position.into(),
            rotation: rotation.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}*/

/*pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}*/

pub trait CameraController {
    fn process_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState);
    fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState);
    fn process_mouse_scroll_event(&mut self, delta: MouseScrollDelta);
    fn process_mouse_move_event(&mut self, dx: f64, dy: f64);
    fn update_camera(&mut self, camera: &mut Camera, dt: Duration);

    fn get_speed(&self) -> f32;
}

#[derive(Debug)]
pub struct FreeCameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    amount_roll_left: f32,
    amount_roll_right: f32,
    mouse_left_pressed: bool,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    scroll_sensitivity: f32,
    pan_sensitivity: f32,
    roll_sensitivity: f32,
}

impl FreeCameraController {
    pub fn new(
        speed: f32,
        scroll_sensitivity: f32,
        pan_sensitivity: f32,
        roll_sensitivity: f32,
    ) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            amount_roll_left: 0.0,
            amount_roll_right: 0.0,
            mouse_left_pressed: false,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            scroll_sensitivity,
            pan_sensitivity,
            roll_sensitivity,
        }
    }
}

impl CameraController for FreeCameraController {
    fn process_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
            }
            VirtualKeyCode::Q => {
                self.amount_roll_left = amount;
            }
            VirtualKeyCode::E => {
                self.amount_roll_right = amount;
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
            }
            _ => {}
        }
    }

    fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => {
                self.mouse_left_pressed = state == ElementState::Pressed;
            }
            _ => {}
        }
    }

    fn process_mouse_scroll_event(&mut self, delta: MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => scroll as f32,
        };
    }

    fn process_mouse_move_event(&mut self, dx: f64, dy: f64) {
        if self.mouse_left_pressed {
            self.rotate_horizontal = dx as f32;
            self.rotate_vertical = dy as f32;
        }
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward, left/right and up/down
        let right = camera.rotation.rotate_vector(Vector3::unit_x());
        let up = camera.rotation.rotate_vector(Vector3::unit_y());
        let forward = camera.rotation.rotate_vector(Vector3::unit_z());
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += -right * (self.amount_right - self.amount_left) * self.speed * dt;
        camera.position += up * (self.amount_up - self.amount_down) * self.speed * dt;

        self.speed += self.speed * self.scroll * self.scroll_sensitivity * dt;
        if self.speed < 0.1 {
            self.speed = 0.1;
        }

        self.scroll = 0.0;

        // Rotate
        let rotation = Quaternion::from(Euler {
            x: Rad(self.rotate_vertical) * self.pan_sensitivity * dt,
            y: Rad(-self.rotate_horizontal) * self.pan_sensitivity * dt,
            z: Rad(self.amount_roll_left - self.amount_roll_right) * self.roll_sensitivity * dt,
        });

        camera.rotation = camera.rotation * rotation;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

    fn get_speed(&self) -> f32 {
        self.speed
    }
}

/*#[derive(Debug)]
pub struct OrbitCameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    orbit_horizontal: f32,
    orbit_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    target: Point3<f32>,
}

impl OrbitCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            orbit_horizontal: 0.0,
            orbit_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            target: (0.0, 0.0, 0.0).into(),
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_left_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.orbit_horizontal = mouse_dx as f32;
        self.orbit_vertical = mouse_dy as f32;
    }

    pub fn process_right_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let forward = camera.rotation.rotate_vector(Vector3::unit_z());
        let right = camera.rotation.rotate_vector(Vector3::unit_x());

        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let scrollward = camera.rotation.rotate_vector(Vector3::unit_z());
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.rotation = camera.rotation
            * Quaternion::from(Euler {
                x: Rad(-self.rotate_vertical) * self.sensitivity * dt,
                y: Rad(self.rotate_horizontal) * self.sensitivity * dt,
                z: Rad(0.0),
            });

        camera.rotation = Quaternion::look_at(
            self.target - camera.position,
            camera.rotation.rotate_vector(Vector3::unit_y()),
        );

        // Rotate
        // Orbit
        let yaw = Rad(self.orbit_horizontal) * self.sensitivity * dt;
        let pitch = Rad(-self.orbit_vertical) * self.sensitivity * dt;

        let rotation = Quaternion::from(Euler {
            x: -pitch,
            y: -yaw,
            z: Rad(0.0),
        });

        let rotation = camera.rotation.conjugate() * rotation * camera.rotation;

        let pos = camera.position - self.target;

        camera.position = self.target + rotation.rotate_vector(pos);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        self.orbit_horizontal = 0.0;
        self.orbit_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        /*if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        }*/
    }
}*/
