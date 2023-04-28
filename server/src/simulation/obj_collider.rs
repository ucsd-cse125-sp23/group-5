use log::debug;
use nalgebra::Point3;
use rapier3d::geometry::ColliderBuilder;
use rapier3d::math::Real;

use tobj;

pub trait FromObject {
    fn from_object_models(models: Vec<tobj::Model>) -> Self;
}

impl FromObject for ColliderBuilder {
    /// Create a collider from a list of object models (combine all the meshes into one collider)
    fn from_object_models(models: Vec<tobj::Model>) -> ColliderBuilder {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0;
        debug!("Loading {} models", models.len());
        for model in models {
            debug!("Model: {:?}", model.name);
            let mesh = &model.mesh;
            vertices.extend(mesh.positions.chunks(3).map(Point3::<Real>::from_slice));

            indices.extend(
                mesh.indices
                    .chunks(3)
                    .map(|i| [i[0], i[1], i[2]].map(|i| i + vertex_offset)),
            );
            vertex_offset = vertices.len() as u32;
        }

        ColliderBuilder::trimesh(vertices, indices)
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::obj_collider::FromObject;
    use approx::relative_eq;
    use rapier3d::geometry::ColliderBuilder;
    use rapier3d::prelude::Isometry;

    #[test]
    fn test_loading_simple_model() {
        // use parent of CARGO_MANIFEST_DIR
        let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::Path::new(&path).parent().unwrap();
        let island = tobj::load_obj(path.join("assets/island.obj"), &tobj::GPU_LOAD_OPTIONS);

        let (models, _materials) = island.unwrap();

        let collider = ColliderBuilder::from_object_models(models);
        let aabb = collider.shape.0.compute_aabb(&Isometry::identity());

        // mins: [-4.934327, -1.3986979, -3.9341192], maxs: [4.454054, 3.599072, 4.615514] }
        relative_eq!(aabb.mins.x, -4.934327);
        relative_eq!(aabb.mins.y, -1.3986979);
        relative_eq!(aabb.mins.z, -3.9341192);
        relative_eq!(aabb.maxs.x, 4.454054);
        relative_eq!(aabb.maxs.y, 3.599072);
        relative_eq!(aabb.maxs.z, 4.615514);
    }
}
