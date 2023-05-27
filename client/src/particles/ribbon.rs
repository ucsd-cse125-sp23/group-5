use nalgebra_glm as glm;
use rand::Rng;
use rand_distr::{Distribution, Normal, Poisson, Uniform};

use crate::particles::Particle;
use crate::particles::gen::ParticleGenerator;
use crate::particles::constants;

pub struct RibbonSection{
    pub pos_1: glm::Vec3,
    pub pos_2: glm::Vec3,
    pub width_1: f32,
    pub width_2: f32,
    pub color: glm::Vec4,
    pub t1: f32,
    pub t2: f32,
    pub tex_id: i32,
    pub z_min: f32, // we'll let the shader calculate z's and clamp it from below
    pub visible_time: f32,
}

impl RibbonSection{
    pub fn to_particle(self) -> Particle{
        Particle{
            start_pos: [self.pos_1[0], self.pos_1[1], self.pos_1[2], self.width_1],
            velocity: [self.pos_1[0], self.pos_1[1], self.pos_1[2], self.width_2],
            color: self.color.into(),
            spawn_time: self.t1,
            size: self.t2,
            tex_id: self.tex_id,
            z_pos: 0.0,
            time_elapsed: 0.0,
            size_growth: 0.0,
            halflife: 0.0,
            FLAG: constants::RIBBON_PARTICLE,
        }
    }
}

// sample from box uniformly for start point
// width is uniform for each streak
pub struct LineRibbonGenerator {
    bounds_min: glm::Vec3,
    bounds_max: glm::Vec3,
    v_dir: glm::Vec3,
    linear_speed: f32,
    linear_variance: f32,
    visible_time: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl LineRibbonGenerator{
    pub fn new(
        bounds_min: glm::Vec3,
        bounds_max: glm::Vec3,
        v_dir: glm::Vec3,
        linear_speed: f32,
        linear_variance: f32,
        visible_time: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self{
        Self{
            bounds_min,
            bounds_max,
            v_dir: glm::normalize(&v_dir),
            linear_speed,
            linear_variance,
            visible_time,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

// implementing without subdividing the lines first
// this may cause draw order/overlap issues
impl ParticleGenerator for LineRibbonGenerator {
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32 {
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        // for positions
        let x_dist = Uniform::new(self.bounds_min[0], self.bounds_max[0]);
        let y_dist = Uniform::new(self.bounds_min[1], self.bounds_max[1]);
        let z_dist = Uniform::new(self.bounds_min[2], self.bounds_max[2]);
        let mut spawn_time = 0.0;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let v = self.v_dir * lin_dist.sample(rng);
            let origin = glm::vec3(
                x_dist.sample(rng),
                y_dist.sample(rng),
                z_dist.sample(rng),
            );
            let end = origin + v * halflife * 2.0;
            let width = size_dist.sample(rng);
            let ribbon = RibbonSection{
                pos_1: origin,
                pos_2: end,
                width_1: width,
                width_2: width,
                color,
                t1: spawn_time,
                t2: halflife * 2.0,
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_min: 0.0,
                visible_time: self.visible_time,
            };
            list.push(ribbon.to_particle());
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list.len() as u32
    }
}