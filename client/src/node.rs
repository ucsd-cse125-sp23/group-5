//use crate::instance::Instance;
use nalgebra_glm;

pub struct Node {
    // pub model: model::Model,
    // pub instances: Vec<Instance>,

    pub childnodes: Vec<Box<Node>>,
    pub childtransforms: Vec<nalgebra_glm::mat4>,
    pub models: Vec<Model>,
    pub modeltransforms: Vec<nalgebra_glm::mat4>,
}