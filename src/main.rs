//! simple 3d wireframe renderer in rust
//! 
//! features:
//! - renders basic 3d shapes (cube, pyramid, octahedron)
//! - perspective projection
//! - rotation animations
//! - keyboard controls (1-3 to switch shapes)
//! 
//! controls:
//! - 1: cube
//! - 2: pyramid
//! - 3: octahedron
//! - esc: exit
//! 
//! written by: John Linthicum
//! created: 2024
//! feel free to mess with the code and do whatever you want with it

use std::f32::consts::PI;
use minifb::{Key, Window, WindowOptions};

/// simple 3d vector
#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
}

/// 4x4 matrix for 3d transforms
#[derive(Clone, Copy, Debug)]
struct Mat4 {
    m: [f32; 16],
}

impl Mat4 {
    fn identity() -> Self {
        Mat4 {
            m: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// matrix multiply: self * other
    fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4 { m: [0.0; 16] };
        for row in 0..4 {
            for col in 0..4 {
                result.m[row * 4 + col] =
                    self.m[row * 4 + 0] * other.m[0 * 4 + col] +
                    self.m[row * 4 + 1] * other.m[1 * 4 + col] +
                    self.m[row * 4 + 2] * other.m[2 * 4 + col] +
                    self.m[row * 4 + 3] * other.m[3 * 4 + col];
            }
        }
        result
    }

    /// make x-axis rotation matrix
    fn rotation_x(angle: f32) -> Mat4 {
        let mut m = Mat4::identity();
        let c = angle.cos();
        let s = angle.sin();
        m.m[5] = c;    // (1,1)
        m.m[6] = -s;   // (1,2)
        m.m[9] = s;    // (2,1)
        m.m[10] = c;   // (2,2)
        m
    }

    /// make y-axis rotation matrix
    fn rotation_y(angle: f32) -> Mat4 {
        let mut m = Mat4::identity();
        let c = angle.cos();
        let s = angle.sin();
        m.m[0] = c;    // (0,0)
        m.m[2] = s;    // (0,2)
        m.m[8] = -s;   // (2,0)
        m.m[10] = c;   // (2,2)
        m
    }

    /// make perspective matrix
    ///
    /// fov: field-of-view in radians
    /// aspect: width / height
    /// near: near plane
    /// far: far plane
    fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let mut m = Mat4::identity();
        let f = 1.0 / (fov / 2.0).tan();
        m.m[0] = f / aspect;
        m.m[5] = f;
        m.m[10] = (far + near) / (near - far);
        m.m[11] = -1.0;
        m.m[14] = (2.0 * far * near) / (near - far);
        m.m[15] = 0.0;
        m
    }

    /// transform vec3 by matrix (w=1)
    fn transform_vec3(&self, v: Vec3) -> Vec3 {
        let x = v.x * self.m[0] + v.y * self.m[4] + v.z * self.m[8] + self.m[12];
        let y = v.x * self.m[1] + v.y * self.m[5] + v.z * self.m[9] + self.m[13];
        let z = v.x * self.m[2] + v.y * self.m[6] + v.z * self.m[10] + self.m[14];
        let w = v.x * self.m[3] + v.y * self.m[7] + v.z * self.m[11] + self.m[15];
        if w != 0.0 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }
}

/// draw line in pixel buffer using bresenham
/// https://www.youtube.com/watch?v=CceepU1vIKo
/// color: 0xRRGGBB int (e.g. 0xffffff for white)
fn draw_line(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
) {
    let mut dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let mut dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let (mut x, mut y) = (x0, y0);

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            buffer[y as usize * width + x as usize] = color;
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// mesh data with vertices and edges
struct Mesh {
    vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
}

/// make cube mesh
fn create_cube() -> Mesh {
    Mesh {
        vertices: vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new( 1.0, -1.0, -1.0),
            Vec3::new( 1.0,  1.0, -1.0),
            Vec3::new(-1.0,  1.0, -1.0),
            Vec3::new(-1.0, -1.0,  1.0),
            Vec3::new( 1.0, -1.0,  1.0),
            Vec3::new( 1.0,  1.0,  1.0),
            Vec3::new(-1.0,  1.0,  1.0),
        ],
        edges: vec![
            (0, 1), (1, 2), (2, 3), (3, 0), // bottom
            (4, 5), (5, 6), (6, 7), (7, 4), // top
            (0, 4), (1, 5), (2, 6), (3, 7), // sides
        ],
    }
}

/// make pyramid mesh
fn create_pyramid() -> Mesh {
    Mesh {
        vertices: vec![
            Vec3::new( 0.0,  1.0,  0.0),    // top
            Vec3::new(-1.0, -1.0, -1.0),    // base
            Vec3::new( 1.0, -1.0, -1.0),
            Vec3::new( 1.0, -1.0,  1.0),
            Vec3::new(-1.0, -1.0,  1.0),
        ],
        edges: vec![
            (1, 2), (2, 3), (3, 4), (4, 1), // base
            (0, 1), (0, 2), (0, 3), (0, 4), // sides
        ],
    }
}

/// make octahedron mesh
fn create_octahedron() -> Mesh {
    Mesh {
        vertices: vec![
            Vec3::new( 0.0,  1.0,  0.0),    // top
            Vec3::new( 0.0, -1.0,  0.0),    // bottom
            Vec3::new(-1.0,  0.0,  0.0),    // middle points
            Vec3::new( 1.0,  0.0,  0.0),
            Vec3::new( 0.0,  0.0, -1.0),
            Vec3::new( 0.0,  0.0,  1.0),
        ],
        edges: vec![
            (0, 2), (0, 3), (0, 4), (0, 5), // top edges
            (1, 2), (1, 3), (1, 4), (1, 5), // bottom edges
            (2, 4), (4, 3), (3, 5), (5, 2), // middle edges
        ],
    }
}

fn main() {
    // window size
    let width = 800;
    let height = 600;

    // create window using minifb
    let mut window = Window::new(
        "3d shapes (1-3 to switch, esc to exit)",
        width,
        height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // limit to ~60 fps
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // make framebuffer
    let mut buffer = vec![0u32; width * height];

    // setup projection matrix
    let aspect_ratio = width as f32 / height as f32;
    let fov = PI / 3.0; // 60 degrees
    let projection = Mat4::perspective(fov, aspect_ratio, 0.1, 100.0);

    // create meshes
    let meshes = [
        create_cube(),
        create_pyramid(),
        create_octahedron(),
    ];
    
    let mut current_mesh = 0;

    let mut angle = 0.0;

    // main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // handle mesh switching
        if window.is_key_down(Key::Key1) { current_mesh = 0; }
        if window.is_key_down(Key::Key2) { current_mesh = 1; }
        if window.is_key_down(Key::Key3) { current_mesh = 2; }

        // clear to black
        for pixel in buffer.iter_mut() {
            *pixel = 0x000000;
        }

        // make rotation matrices
        let rx = Mat4::rotation_x(angle * 1.3);
        let ry = Mat4::rotation_y(angle);

        // combine transforms
        let model = rx.mul(&ry);

        // get current mesh
        let mesh = &meshes[current_mesh];

        // draw mesh edges
        for &(i0, i1) in &mesh.edges {
            let v0 = mesh.vertices[i0];
            let v1 = mesh.vertices[i1];

            // rotate and move to camera space
            let v0_transformed = model.transform_vec3(v0);
            let v1_transformed = model.transform_vec3(v1);

            let v0_in_world = Vec3::new(v0_transformed.x, v0_transformed.y, v0_transformed.z - 5.0);
            let v1_in_world = Vec3::new(v1_transformed.x, v1_transformed.y, v1_transformed.z - 5.0);

            // project to screen
            let p0 = projection.transform_vec3(v0_in_world);
            let p1 = projection.transform_vec3(v1_in_world);

            // convert to screen coords
            let x0 = ((p0.x + 1.0) * 0.5 * width as f32) as i32;
            let y0 = ((1.0 - p0.y) * 0.5 * height as f32) as i32;
            let x1 = ((p1.x + 1.0) * 0.5 * width as f32) as i32;
            let y1 = ((1.0 - p1.y) * 0.5 * height as f32) as i32;

            // draw edge
            draw_line(&mut buffer, width, height, x0, y0, x1, y1, 0xffffff);
        }

        // update screen
        window
            .update_with_buffer(&buffer, width, height)
            .unwrap();

        // increment rotation
        angle += 0.02;
    }
}
