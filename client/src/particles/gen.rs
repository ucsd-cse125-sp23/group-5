extern crate nalgebra_glm as glm;
use rand::Rng;
use rand_distr::{Distribution, Geometric, LogNormal, Normal, Poisson, Uniform};
use std::f32::consts::{FRAC_PI_2, PI};

use crate::particles::constants::POINT_PARTICLE;
use crate::particles::Particle;

pub trait ParticleGenerator {
    ////
    /// list: vector to place generated particles in
    /// spawning time: amount of time to keep spawning
    /// spawn rate: average rate of spawning in particles per second
    /// num_textures: number of possible particle textures in one file (arranged vertically)
    /// returns: number of particles generated
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> f32;
}

pub struct SphereGenerator {
    source: glm::Vec3,
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl SphereGenerator {
    pub fn new(
        source: glm::Vec3,
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        Self {
            source,
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for SphereGenerator {
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
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let dir_dist = Normal::new(0.0, 1.0).unwrap();
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        // let v = self.dir;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let lin_scale = lin_dist.sample(rng);
            let dir = glm::vec3(
                dir_dist.sample(rng),
                dir_dist.sample(rng),
                dir_dist.sample(rng),
            );
            let v = glm::normalize(&dir);
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [
                    v[0] * lin_scale,
                    v[1] * lin_scale,
                    v[2] * lin_scale,
                    ang_dist.sample(rng),
                ],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}

pub struct ConeGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    up: glm::Vec3,
    right: glm::Vec3,
    r: f32, // radius of circle created by spread
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl ConeGenerator {
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        up: glm::Vec3,
        spread: f32, // in radians
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        let right = glm::normalize(&glm::cross(&dir, &up));
        let half_spread = spread / 2.0;
        let r = half_spread.tan();
        Self {
            source,
            dir: glm::normalize(&dir),
            up: glm::normalize(&glm::cross(&right, &dir)),
            right,
            // half degree in radians = degree / 2 * 2pi / 360
            //  = degree * pi / 180 = degree * (pi/2) * 360
            r,
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for ConeGenerator {
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
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let dir_r_dist = Uniform::new(0.0, self.r);
        let dir_theta_dist = Uniform::new(0.0, PI * 2.0);
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        // let v = self.dir;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let lin_scale = lin_dist.sample(rng);
            let r = dir_r_dist.sample(rng).sqrt();
            let theta = dir_theta_dist.sample(rng);
            let v: glm::Vec3 = self.dir + r * theta.cos() * self.right + r * theta.sin() * self.up;
            let v = glm::normalize(&v);
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [
                    v[0] * lin_scale,
                    v[1] * lin_scale,
                    v[2] * lin_scale,
                    ang_dist.sample(rng),
                ],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}

pub struct CylinderGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    up: glm::Vec3,
    r: f32, // max radius of base of cylinder
    half_height: f32, // half height of cylinder
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl CylinderGenerator {
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        up: glm::Vec3,
        r: f32, 
        half_height: f32,
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        Self {
            source,
            dir: glm::normalize(&dir),
            up: glm::normalize(&up),
            // half degree in radians = degree / 2 * 2pi / 360
            //  = degree * pi / 180 = degree * (pi/2) * 360
            r,
            half_height,
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for CylinderGenerator {
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
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let dir_r_dist = Uniform::new(self.r / 2.0, self.r);
        let dir_theta_dist = Uniform::new(0.0, PI * 2.0);
        let dir_y_dist = Uniform::new(-self.half_height, self.half_height);
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        // let v = self.dir;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let lin_scale = lin_dist.sample(rng);
            let r = dir_r_dist.sample(rng).sqrt();
            let theta = dir_theta_dist.sample(rng);
            let y = dir_y_dist.sample(rng);
            let source = self.source + glm::vec3(r * theta.cos(), y, r * theta.sin());
            let v: glm::Vec3 = self.dir;
            list.push(Particle {
                start_pos: [source[0], source[1], source[2], 0.0],
                color: color.into(),
                velocity: [
                    v[0] * lin_scale,
                    v[1] * lin_scale,
                    v[2] * lin_scale,
                    0.0,
                ],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}

pub struct FanGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    up: glm::Vec3,
    right: glm::Vec3,
    half_spread: f32, // in radians
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl FanGenerator {
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        up: glm::Vec3,
        spread: f32, // in degrees
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        let right = glm::normalize(&glm::cross(&dir, &up));
        Self {
            source,
            dir: glm::normalize(&dir),
            up: glm::normalize(&glm::cross(&right, &dir)),
            right,
            // half degree in radians = degree / 2 * 2pi / 360
            //  = degree * pi / 180 = degree * (pi/2) * 360
            half_spread: (spread) / 360.0 * FRAC_PI_2,
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for FanGenerator {
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
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let ang_dir = Uniform::new(-self.half_spread, self.half_spread);
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        // let v = self.dir;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let lin_scale = lin_dist.sample(rng);
            let angle = ang_dir.sample(rng);
            let v: glm::Vec3 = (angle.cos() * self.dir + angle.sin() * self.right) * lin_scale;
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [v[0], v[1], v[2], ang_dist.sample(rng)],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}

pub struct LineGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl LineGenerator {
    //// Line particle generator
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        Self {
            source,
            dir: glm::normalize(&dir),
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for LineGenerator {
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
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let v = self.dir * lin_dist.sample(rng);
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [v[0], v[1], v[2], ang_dist.sample(rng)],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}

pub struct RainGenerator {
    source: glm::Vec3,     // Central point of the rain area
    area: (f32, f32, f32), // 3d box of the rain area
    dir: glm::Vec3,        // Direction of rain (usually down)
    linear_speed: f32,
    linear_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl RainGenerator {
    pub fn new(
        source: glm::Vec3,
        area: (f32, f32, f32),
        dir: glm::Vec3,
        linear_speed: f32,
        linear_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        Self {
            source,
            area,
            dir: glm::normalize(&dir),
            linear_speed,
            linear_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for RainGenerator {
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
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        let x_dist = Uniform::new(-self.area.0 / 2.0, self.area.0 / 2.0);
        let z_dist = Uniform::new(-self.area.1 / 2.0, self.area.1 / 2.0);
        let y_dist = Uniform::new(-self.area.2 / 2.0, self.area.2 / 2.0);

        let v_dist = LogNormal::new(self.linear_speed, self.linear_variance).unwrap();

        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let v = self.dir * v_dist.sample(rng);
            let pos =
                self.source + glm::vec3(x_dist.sample(rng), y_dist.sample(rng), z_dist.sample(rng));
            list.push(Particle {
                start_pos: [pos[0], pos[1], pos[2], 0.0],
                color: color.into(),
                velocity: [v[0], v[1], v[2], 0.0],
                normal_1: [0., 0., 0., 0.],
                normal_2: [0., 0., 0., 0.],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as i32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                FLAG: POINT_PARTICLE,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        list[list.len() - 1].spawn_time + halflife * 2.0
    }
}
