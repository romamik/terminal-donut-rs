use crossterm::{cursor, execute};
use glam::{Mat4, Vec3, vec3};

const MAX_STEPS: i32 = 100;
const MAX_DISTANCE: f32 = 100.0;
const EPSILON: f32 = 0.01;
const SYMBOLS: &[u8] = b" .,:;i1tfLCG08@";

pub trait Sdf {
    fn distance(&self, pt: Vec3) -> f32;
}

pub struct SdfSphere {
    pub center: Vec3,
    pub radius: f32,
}
impl Sdf for SdfSphere {
    fn distance(&self, pt: Vec3) -> f32 {
        (pt - self.center).length() - self.radius
    }
}

pub struct SdfBox {
    pub center: Vec3,
    pub half_size: Vec3,
}

impl Sdf for SdfBox {
    fn distance(&self, pt: Vec3) -> f32 {
        let p = (pt - self.center).abs() - self.half_size;
        let outside_distance = p.max(Vec3::ZERO).length();
        let inside_distance = p.x.max(p.y).max(p.z).min(0.0);

        outside_distance + inside_distance
    }
}

pub struct Transform<Inner> {
    pub mat: Mat4,
    pub inner: Inner,
}

impl<Inner: Sdf> Sdf for Transform<Inner> {
    fn distance(&self, pt: Vec3) -> f32 {
        self.inner.distance((self.mat * pt.extend(1.0)).truncate())
    }
}

fn estimate_normal(scene: &impl Sdf, p: Vec3) -> Vec3 {
    let eps = 0.0001;
    let dx = eps * Vec3::X;
    let dy = eps * Vec3::Y;
    let dz = eps * Vec3::Z;
    let normal = vec3(
        scene.distance(p + dx) - scene.distance(p - dx),
        scene.distance(p + dy) - scene.distance(p - dy),
        scene.distance(p + dz) - scene.distance(p - dz),
    );
    normal.normalize()
}

fn lambert_shading(normal: Vec3, light_dir: Vec3) -> f32 {
    normal.dot(light_dir).max(0.0)
}

fn cast_ray(scene: &impl Sdf, start: Vec3, ray: Vec3, light_dir: Vec3) -> f32 {
    let mut step = 0;
    let mut total_distance_traveled = 0.0;

    let mut current_point = start;
    while step < MAX_STEPS && total_distance_traveled < MAX_DISTANCE {
        let current_distance = scene.distance(current_point);
        if current_distance < EPSILON {
            let normal = estimate_normal(scene, current_point);
            let shading = lambert_shading(normal, light_dir);
            return 0.1 + shading * 0.9;
        }
        total_distance_traveled += current_distance;
        current_point += ray * (current_distance);
        step += 1;
    }

    0.0 // Pixel is in empty space
}

pub trait Output {
    fn reset(&self);
    fn size(&self) -> (usize, usize);
    fn aspect(&self) -> f32;
    fn print_char(&self, c: u8);
    fn println(&self);
}

pub fn render_scene(
    scene: &impl Sdf,
    camera: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
    camera_size: f32,
    light_dir: Vec3,
    output: &impl Output,
) {
    let forward = (camera_look_at - camera).normalize_or(Vec3::NEG_Z);
    let right = forward.cross(camera_up).normalize_or(Vec3::X);
    let up = forward.cross(right).normalize_or(Vec3::NEG_Y);

    let (screen_w, screen_h) = output.size();

    let (camera_width, camera_height) = if screen_w > screen_h {
        (
            camera_size * screen_w as f32 / screen_h as f32 * output.aspect(),
            camera_size,
        )
    } else {
        (
            camera_size,
            camera_size * screen_h as f32 / screen_w as f32 / output.aspect(),
        )
    };

    for screen_y in 0..screen_h {
        if screen_y != 0 {
            output.println();
        }
        for screen_x in 0..screen_w {
            let x = camera + right * (camera_width * (screen_x as f32 / screen_w as f32 - 0.5));
            let y = camera + up * (camera_height * (screen_y as f32 / screen_h as f32 - 0.5));
            let intensity = cast_ray(scene, x + y, forward, light_dir);
            let char_index = ((intensity.clamp(0.0, 1.0) * (SYMBOLS.len() as f32)) as usize)
                .clamp(0, SYMBOLS.len() - 1);
            output.print_char(SYMBOLS[char_index]);
        }
    }
}

struct CrosstermOutput;

impl Output for CrosstermOutput {
    fn reset(&self) {
        execute!(std::io::stdout(), cursor::MoveTo(0, 0)).unwrap();
    }

    fn aspect(&self) -> f32 {
        0.5
    }

    fn size(&self) -> (usize, usize) {
        let (w, h) = crossterm::terminal::size().unwrap();
        (w as usize, h as usize)
    }

    fn print_char(&self, c: u8) {
        print!("{}", c as char)
    }

    fn println(&self) {
        println!()
    }
}

fn main() {
    // let scene = SdfSphere {
    //     center: Vec3::ZERO,
    //     radius: 5.0,
    // };
    let scene = SdfBox {
        center: Vec3::ZERO,
        half_size: vec3(5.0, 2.0, 3.0),
    };
    render_scene(
        &scene,
        vec3(10.0, 10.0, 10.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        20.0,
        vec3(0.0, 1.0, 0.0),
        &CrosstermOutput,
    );
}
