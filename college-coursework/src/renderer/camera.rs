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

/// Position and transformation matrix of the camera for the GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        //! Create a new CameraUniform
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        //! Update the view projection transformation matrix

        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

/// Container to store the position of the camera in the Entity Component System
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct CameraPosition(pub Point3<f32>);
impl Default for CameraPosition {
    fn default() -> Self {
        Self(Point3::new(0.0, 0.0, 0.0))
    }
}

/// Container to store the speed of the camera in the Entity Component System
#[derive(Debug, Component, Default)]
#[storage(VecStorage)]
pub struct CameraSpeed(pub f32);

/// Data structure that stores the position and rotation of the camera
#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    rotation: Quaternion<f32>,
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>, R: Into<Quaternion<f32>>>(position: P, rotation: R) -> Self {
        //! Create a new camera

        Self {
            position: position.into(),
            rotation: rotation.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        //! Get the transformation matrix of the camera

        Matrix4::look_at_rh(
            self.position,
            self.position + self.rotation.rotate_vector(Vector3::unit_z()),
            self.rotation.rotate_vector(Vector3::unit_y()),
        )
    }
}

/// Projection of the camera
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}
impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        //! Create a new projection

        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        //! Resize the projection to match the new window
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        //! Return the transformation matrix
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

/// Trait representing a camera controller, will be used more in later iterations for multiple camera types
pub trait CameraController {
    fn process_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState);
    fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState);
    fn process_mouse_scroll_event(&mut self, delta: MouseScrollDelta);
    fn process_mouse_move_event(&mut self, dx: f64, dy: f64);
    fn update_camera(&mut self, camera: &mut Camera, dt: Duration);

    fn get_speed(&self) -> f32;
    fn set_speed(&mut self, speed: f32);
}

/// Controller for a free camera
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
        //! Create a new free camera controller

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
        //! Handle keyboard input

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
        //! Handle mouse click

        match button {
            MouseButton::Left => {
                self.mouse_left_pressed = state == ElementState::Pressed;
            }
            _ => {}
        }
    }

    fn process_mouse_scroll_event(&mut self, delta: MouseScrollDelta) {
        //! Handle scrolling

        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => scroll as f32,
        };
    }

    fn process_mouse_move_event(&mut self, dx: f64, dy: f64) {
        //! Handle mouse moved

        if self.mouse_left_pressed {
            self.rotate_horizontal = dx as f32;
            self.rotate_vertical = dy as f32;
        }
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        //! Update the camera using the camera controller

        let dt = dt.as_secs_f32();

        // Move forward/backward, left/right and up/down
        let right = camera.rotation.rotate_vector(Vector3::unit_x());
        let up = camera.rotation.rotate_vector(Vector3::unit_y());
        let forward = camera.rotation.rotate_vector(Vector3::unit_z());
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += -right * (self.amount_right - self.amount_left) * self.speed * dt;
        camera.position += up * (self.amount_up - self.amount_down) * self.speed * dt;

        self.speed *= 2_f32.powf(self.scroll * 1e-2 * self.scroll_sensitivity);
        if self.speed < 0.1 {
            self.speed = 0.1;
        }

        self.scroll = 0.0;

        // Rotate the camera yaw, pitch and roll
        let rotation = Quaternion::from(Euler {
            x: Rad(self.rotate_vertical) * self.pan_sensitivity * dt,
            y: Rad(-self.rotate_horizontal) * self.pan_sensitivity * dt,
            z: Rad(self.amount_roll_left - self.amount_roll_right) * self.roll_sensitivity * dt,
        });

        camera.rotation = camera.rotation * rotation;

        // Prevent the camera from continuously truning when it is not wanted
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

    fn get_speed(&self) -> f32 {
        //! Returns the speed of the camera

        self.speed
    }

    fn set_speed(&mut self, speed: f32) {
        //! Sets the speed of the camera

        self.speed = speed;
    }
}
