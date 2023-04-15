use nalgebra::Point3;
use rapier3d::geometry::ColliderBuilder;
use rapier3d::math::Real;
use tobj;

pub trait FromObject {
    fn from_object(models: Vec<tobj::Model>) -> Self;
}

impl FromObject for ColliderBuilder {
    fn from_object(models: Vec<tobj::Model>) -> ColliderBuilder {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0;
        for model in models {
            let mesh = &model.mesh;
            vertices.extend(mesh
                .positions
                .chunks(3)
                .map(Point3::<Real>::from_slice)
            );

            indices.extend(mesh
                .indices
                .chunks(3)
                .map(|i| [i[0], i[1], i[2]].map(|i| i + vertex_offset))
            );
            vertex_offset += vertices.len() as u32;
        }

        ColliderBuilder::trimesh(vertices, indices)
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Point, Point3};
    use rapier3d::geometry::ColliderBuilder;
    use rapier3d::math::Real;
    use rapier3d::prelude::Isometry;
    use crate::simulation::obj_collider::{FromObject};

    #[test]
    fn test() {
        let cornell_box = tobj::load_obj("assets/unit_cube_divided_dup.obj", &tobj::GPU_LOAD_OPTIONS);

        let (models, materials) = cornell_box.unwrap();

        let collider = ColliderBuilder::from_object(models);
        let bounding_sphere = collider.shape.0.compute_bounding_sphere(&Isometry::identity());
        let radius = bounding_sphere.radius();
        let center = bounding_sphere.center();
        assert!(radius > 0.86);
        assert_ne!(center, &Point3::<Real>::origin());
    }
}
