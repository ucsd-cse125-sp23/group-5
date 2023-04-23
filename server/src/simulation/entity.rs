use rapier3d::prelude::*;

// TODO: decide the types of entities and maybe make it into an enum
pub type Entity = u32;

pub struct EntityHandles {
    pub rigid_body: Option<RigidBodyHandle>,
    pub collider: Option<ColliderHandle>,
}
