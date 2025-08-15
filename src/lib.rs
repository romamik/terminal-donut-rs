use std::fmt::Write;

use glam::{Mat4, Vec3, vec3};

const MAX_STEPS: i32 = 100;
const MAX_DISTANCE: f32 = 100.0;
const EPSILON: f32 = 0.01;
const SYMBOLS: &[u8] = b" .,:;i1tfLCG08@";

pub trait Sdf {
    fn distance(&self, pt: Vec3) -> f32;

    fn boxed(self) -> Box<dyn Sdf>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<I, T> Sdf for I
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
    T: Sdf,
{
    fn distance(&self, pt: Vec3) -> f32 {
        let mut distance = f32::MAX;
        for inner in self.into_iter() {
            distance = distance.min(inner.distance(pt))
        }
        distance
    }
}

impl Sdf for Box<dyn Sdf> {
    fn distance(&self, pt: Vec3) -> f32 {
        self.as_ref().distance(pt)
    }
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

pub struct SdfDonut {
    pub center: Vec3,
    pub radius: f32,
    pub tube_radius: f32,
}

impl Sdf for SdfDonut {
    fn distance(&self, pt: Vec3) -> f32 {
        let p = pt - self.center;
        let q = glam::Vec2::new((p.x * p.x + p.y * p.y).sqrt() - self.radius, p.z);
        q.length() - self.tube_radius
    }
}

pub struct SdfTransform<Inner> {
    pub mat: Mat4,
    pub inner: Inner,
}

impl<Inner: Sdf> Sdf for SdfTransform<Inner> {
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
    normal.dot(-light_dir).max(0.0)
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
    fn size(&self) -> (usize, usize);
    fn aspect(&self) -> f32;
    fn present(&self, frame: &str);
}

pub fn render_scene(
    scene: &Scene,
    screen_width: usize,
    screen_height: usize,
    screen_aspect: f32,
) -> String {
    let forward = (scene.camera_up - scene.camera_pos).normalize_or(Vec3::NEG_Z);
    let right = forward.cross(scene.camera_up).normalize_or(Vec3::X);
    let up = forward.cross(right).normalize_or(Vec3::NEG_Y);

    let (camera_width, camera_height) = if screen_width > screen_height {
        (
            scene.camera_size * screen_width as f32 / screen_height as f32 * screen_aspect,
            scene.camera_size,
        )
    } else {
        (
            scene.camera_size,
            scene.camera_size * screen_height as f32 / screen_width as f32 / screen_aspect,
        )
    };

    let mut buffer = String::with_capacity((screen_width + 1) * screen_height);
    for screen_y in 0..screen_height {
        if screen_y != 0 {
            // buffer.write_char('\n').unwrap()
        }
        for screen_x in 0..screen_width {
            let x = scene.camera_pos
                + right * (camera_width * (screen_x as f32 / screen_width as f32 - 0.5));
            let y = scene.camera_pos
                + up * (camera_height * (screen_y as f32 / screen_height as f32 - 0.5));
            let intensity = cast_ray(&scene.scene, x + y, forward, scene.light_dir);
            let char_index = ((intensity.clamp(0.0, 1.0) * (SYMBOLS.len() as f32)) as usize)
                .clamp(0, SYMBOLS.len() - 1);
            buffer.write_char(SYMBOLS[char_index] as char).unwrap();
        }
    }

    buffer
}

pub struct Scene {
    pub scene: Box<dyn Sdf>,
    pub camera_pos: Vec3,
    pub look_at: Vec3,
    pub camera_up: Vec3,
    pub camera_size: f32,
    pub light_dir: Vec3,
}

pub fn scene(time: f32) -> Scene {
    Scene {
        scene: SdfTransform {
            mat: Mat4::from_rotation_x(time) * Mat4::from_rotation_y(time),
            inner: [
                SdfSphere {
                    center: Vec3::ZERO,
                    radius: 7.0,
                }
                .boxed(),
                SdfBox {
                    center: vec3(f32::sin(time * 2.0) * 3.0, 0.0, 0.0),
                    half_size: vec3(10.0, 3.0, 3.0),
                }
                .boxed(),
                SdfDonut {
                    center: Vec3::ZERO,
                    radius: 10.0,
                    tube_radius: 2.0,
                }
                .boxed(),
            ],
        }
        .boxed(),
        camera_pos: vec3(0.0, 0.0, 20.0),
        look_at: Vec3::ZERO,
        camera_up: vec3(0.0, 1.0, 0.0),
        camera_size: 20.0,
        light_dir: vec3(1.0, -1.0, -1.0),
    }
}
