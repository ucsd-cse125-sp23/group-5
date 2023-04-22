use crate::instance;
use crate::model;

pub struct Scene {
    pub objects: Vec<model::Model>,
    pub instance_vectors: Vec<Vec<instance::Instance>>,
}
