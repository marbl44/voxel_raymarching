extern crate glium;
extern crate winit;

use glium::{
    backend::glutin,
    implement_vertex, index, program,
    texture::{ClientFormat, RawImage3d, Texture3d},
    uniform, Surface,
};
use json::read_json_file;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoopBuilder,
    keyboard::{KeyCode, PhysicalKey},
};

mod json;

const SIZE: [f32; 2] = [1280.0, 720.0];

const WIDTH: usize = 128;
const HEIGHT: usize = 10;
const LENGTH: usize = 128;

#[derive(Debug)]
struct Camera {
    pos: [f32; 3],
    look_at: [f32; 3],
    pitch: f32,
    yaw: f32,
}

impl Camera {
    fn new(pos: [f32; 3]) -> Self {
        let mut camera = Self {
            pos,
            look_at: [0.0; 3],
            pitch: 0.0,
            yaw: 0.0,
        };
        camera.update_look_at();
        camera
    }

    fn update_look_at(&mut self) {
        let front = [
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ];

        let length = (front[0] * front[0] + front[1] * front[1] + front[2] * front[2]).sqrt();
        let front = [front[0] / length, front[1] / length, front[2] / length];

        self.look_at = [
            self.pos[0] + front[0],
            self.pos[1] + front[1],
            self.pos[2] + front[2],
        ];
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VoxelData {
    data: Vec<f32>,
}

fn index(pos: [usize; 3]) -> usize {
    (pos[0] + pos[1] * WIDTH + pos[2] * WIDTH * HEIGHT) * 4
}

impl Default for VoxelData {
    fn default() -> Self {
        let mut data = vec![1.0; (WIDTH * HEIGHT * LENGTH) * 4];
        let mut rng = rand::thread_rng();

        for i in 0..WIDTH {
            for j in 0..LENGTH {
                data[index([i, 0, j]) + 3] = 0.0;
            }
        }

        data.iter_mut().enumerate().for_each(|(i, value)| {
            if !(i % 4 == 3) {
                let random: f32 = rng.r#gen();
                *value = random;
            }
        });

        Self { data }
    }
}

impl VoxelData {
    fn new() -> Self {
        match read_json_file("data.json") {
            Ok(data) => data,
            Err(_) => VoxelData::default(),
        }
    }

    fn texture(&self) -> RawImage3d<f32> {
        RawImage3d {
            data: std::borrow::Cow::Borrowed(&self.data),
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth: LENGTH as u32,
            format: ClientFormat::F32F32F32F32,
        }
    }
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .build()
        .expect("Error building event loop.");
    let (window, display) = glutin::SimpleWindowBuilder::new()
        .with_inner_size(SIZE[0] as u32, SIZE[1] as u32)
        .with_title("raymarching voxels")
        .build(&event_loop);
    window.set_cursor_visible(false);

    let program = program::Program::from_source(
        &display,
        &fs::read_to_string("programs/vertex.glsl").unwrap(),
        &fs::read_to_string("programs/fragment.glsl").unwrap(),
        None,
    )
    .expect("Invalid shader.");

    let shape = vec![
        Vertex {
            position: [-1.0, -1.0],
        },
        Vertex {
            position: [1.0, -1.0],
        },
        Vertex {
            position: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
        },
        Vertex {
            position: [-1.0, -1.0],
        },
    ];
    let indices = index::NoIndices(index::PrimitiveType::TrianglesList);
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let voxels = VoxelData::new();
    let mut fps = FPS::new();
    let mut camera = Camera::new([0.0, HEIGHT as f32 + 3.0, 0.0]);
    let mut keys: HashMap<KeyCode, bool> = HashMap::new();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let front = [
                    camera.yaw.cos() * camera.pitch.cos(),
                    camera.pitch.sin(),
                    camera.yaw.sin() * camera.pitch.cos(),
                ];
                let up = [0.0, 1.0, 0.0];
                let right = [
                    up[1] * front[2] - up[2] * front[1],
                    up[2] * front[0] - up[0] * front[2],
                    up[0] * front[1] - up[1] * front[0],
                ];
                let cam_speed = 10.0;
                if let Some(pressed) = keys.get(&KeyCode::KeyW) {
                    if *pressed {
                        camera.pos[0] += front[0] / cam_speed;
                        camera.pos[1] += front[1] / cam_speed;
                        camera.pos[2] += front[2] / cam_speed;
                    }
                }
                if let Some(pressed) = keys.get(&KeyCode::KeyA) {
                    if *pressed {
                        camera.pos[0] -= right[0] / cam_speed;
                        camera.pos[1] -= right[1] / cam_speed;
                        camera.pos[2] -= right[2] / cam_speed;
                    }
                }
                if let Some(pressed) = keys.get(&KeyCode::KeyS) {
                    if *pressed {
                        camera.pos[0] -= front[0] / cam_speed;
                        camera.pos[1] -= front[1] / cam_speed;
                        camera.pos[2] -= front[2] / cam_speed;
                    }
                }
                if let Some(pressed) = keys.get(&KeyCode::KeyD) {
                    if *pressed {
                        camera.pos[0] += right[0] / cam_speed;
                        camera.pos[1] += right[1] / cam_speed;
                        camera.pos[2] += right[2] / cam_speed;
                    }
                }
                if let Some(pressed) = keys.get(&KeyCode::Space) {
                    if *pressed {
                        camera.pos[1] += 1.0 / cam_speed;
                    }
                }
                if let Some(pressed) = keys.get(&KeyCode::ShiftLeft) {
                    if *pressed {
                        camera.pos[1] -= 1.0 / cam_speed;
                    }
                }
                camera.update_look_at();

                let mut target = display.draw();
                target
                    .draw(
                        &vertex_buffer,
                        &indices,
                        &program,
                        &uniform! {
                            u_resolution: SIZE,
                            u_texture: Texture3d::new(&display, voxels.texture()).unwrap(),
                            u_cam_pos: camera.pos,
                            u_cam_look_at: camera.look_at,
                            u_yaw: camera.yaw,
                            u_pitch: camera.pitch,
                        },
                        &Default::default(),
                    )
                    .unwrap();

                target.finish().expect("Failed to draw.");
                fps.calculate();
                window.set_title(&format!("{}", fps.fps.floor()));
            }
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                    },
                ..
            } => {
                if !window.has_focus() {
                    window.set_cursor_visible(true);
                    return;
                } else {
                    window.set_cursor_visible(false);
                }
                let center = PhysicalPosition::new(
                    window.inner_size().width as f64 / 2.0,
                    window.inner_size().height as f64 / 2.0,
                );
                let delta = [center.x - position.x, center.y - position.y];
                camera.yaw += delta[0] as f32 / 800.0;
                camera.pitch += delta[1] as f32 / 800.0;

                window.set_cursor_position(center).unwrap();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key,
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => match physical_key {
                PhysicalKey::Code(code) => {
                    keys.insert(code, state.is_pressed());
                }
                _ => {}
            },
            _ => {}
        })
        .expect("Error running loop.");
}

struct FPS {
    frame_count: u32,
    last_fps_update: Instant,
    fps: f64,
}

impl FPS {
    fn new() -> Self {
        Self {
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 0.0,
        }
    }

    fn calculate(&mut self) {
        self.frame_count += 1;
        let current = Instant::now();
        let elapsed = current.duration_since(self.last_fps_update);

        if elapsed >= Duration::new(1, 0) {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();

            self.frame_count = 0;
            self.last_fps_update = current;
        }
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);
