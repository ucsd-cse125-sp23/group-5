use nalgebra_glm as glm;
use rand::Rng;
use rand_distr::{Distribution, Normal, Poisson, Uniform};

use crate::particles::constants;
use crate::particles::gen::ParticleGenerator;
use crate::particles::Particle;

#[derive(Copy, Clone, Debug)]
pub struct TrailSection {
    pub pos_1: glm::Vec3,
    pub pos_2: glm::Vec3,
    pub pos_3: glm::Vec3,
    pub pos_4: glm::Vec3,
    pub color: glm::Vec4,
    pub t1: f32,
    pub t2: f32,
    pub tex_id: i32,
    pub z_max: f32, // we'll let the shader calculate z's and clamp it from below
    pub visible_time: f32,
}

impl TrailSection {
    pub fn to_particle(self) -> Particle {
        Particle {
            start_pos: [self.pos_1[0], self.pos_1[1], self.pos_1[2], 1.0],
            velocity: [self.pos_2[0], self.pos_2[1], self.pos_2[2], 1.0],
            color: self.color.into(),
            normal_1: [self.pos_3[0], self.pos_3[1], self.pos_3[2], 1.0],
            normal_2: [self.pos_4[0], self.pos_4[1], self.pos_4[2], 1.0],
            spawn_time: self.t1,
            size: self.t2,
            tex_id: self.tex_id,
            z_pos: 0.0,
            time_elapsed: 0.0,
            size_growth: 0.0,
            halflife: self.visible_time,
            FLAG: constants::TRAIL_PARTICLE,
        }
    }
}

// the entire trail is traversed in a lifetime
pub struct PathTrailGenerator {
    path: Vec<glm::Vec3>,
    times: Vec<f32>, // between 0 and 1
    width: Vec<f32>,
    normal: glm::Vec3,
    subdivisions: u32,
    visible_time: f32,
}

// only for generating one thing
impl PathTrailGenerator {
    pub fn new(
        path: Vec<glm::Vec3>,
        times: Vec<f32>, // between 0 and 1
        width: Vec<f32>,
        normal: glm::Vec3,
        subdivisions: u32,
        visible_time: f32,
    ) -> Self {
        Self {
            path,
            times,
            width,
            normal: glm::normalize(&normal),
            subdivisions,
            visible_time,
        }
    }
}

impl ParticleGenerator for PathTrailGenerator {
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> f32 {
        let total_time = halflife * 2.0;
        for i in 0..(self.path.len() - 1) {
            let tangent = self.path[i + 1] - self.path[i];
            let section_time =
                total_time * (self.times[i + 1] - self.times[i]) / (self.subdivisions as f32);
            for div in 0..self.subdivisions {
                let p0 = self.path[i] + (div as f32) / (self.subdivisions as f32) * tangent;
                let p1 = self.path[i] + ((div + 1) as f32) / (self.subdivisions as f32) * tangent;
                let w0 = self.width[i]
                    + (div as f32) / (self.subdivisions as f32)
                        * (self.width[i + 1] - self.width[i]);
                let w1 = self.width[i]
                    + ((div + 1) as f32) / (self.subdivisions as f32)
                        * (self.width[i + 1] - self.width[i]);
                let trail = TrailSection {
                    pos_1: (p0 - 0.01 * w0 * self.normal),
                    pos_2: (p0 + 0.01 * w0 * self.normal),
                    pos_3: (p1 - 0.01 * w1 * self.normal),
                    pos_4: (p1 + 0.01 * w1 * self.normal),
                    color,
                    t1: self.times[i] + (div as f32) * section_time,
                    t2: self.times[i] + (div as f32 + 1.0) * section_time,
                    tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                    z_max: 0.0,
                    visible_time: self.visible_time,
                };
                //println!("trail piece: {:?}", trail);
                list.push(trail.to_particle());
            }
        }
        list[list.len() - 1].spawn_time + self.visible_time
    }
}
