use crate::camera::Camera;
use crate::{instance, resources};
use crate::instance::Instance;
use crate::model::{self, InstancedModel, Model};
use std::collections::HashMap;
use std::cell::Cell;

use nalgebra_glm as glm;

pub enum ModelIndices{
    ISLAND = 0,
    PLAYER = 1,
    CUBE = 2,
    FERRIS = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelIndex{
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub childnodes: Vec<String>,
    pub models: Vec<(ModelIndex, Instance)>,
}

impl Node{
    pub fn new() -> Self {
        Node {
            childnodes: Vec::new(),
            models: Vec::new(),
        }
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scene{
    pub objects: Vec<model::Model>,
    pub scene_graph: HashMap<String,(Node,Instance)>,
    pub objects_and_instances: HashMap<ModelIndex, Vec<Instance>>,
}

impl Scene{
    pub fn new(objs: Vec<model::Model>) -> Self {
        Scene {
            objects: objs,
            scene_graph: HashMap::new(),
            objects_and_instances: HashMap::new(),
        }
    }

    pub fn draw_scene_dfs(&mut self, camera: &Camera){ // get the view matrix from the camera
        self.objects_and_instances.clear();
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0
        );

        // stacks needed for DFS: 
        let mut dfs_stack: Vec<&Node> = Vec::new();
        let mut matrix_stack: Vec<glm::TMat4<f32>> = Vec::new(); 

        // state needed for DFS:
        let mut cur_node: &Node = &self.scene_graph.get("world").unwrap().0; // self.scene_graph.get("world").unwrap(); // should be the root of the tree to start --> "world"
        let mut cur_VM: glm::TMat4<f32> = mat4_identity; //camera.calc_matrix(); // should be the camera's view matrix to start --> "world"s modelview matrix is the camera's view matrix
                                                                    // currently it's just the identity matrix!
        dfs_stack.push(cur_node);
        matrix_stack.push(cur_VM);

        // let mut total_number_of_edges: usize = 0;
        // for n in self.scene_graph.iter() {
        //    total_number_of_edges += n.1.childnodes.len();
        // }

        // println!("total number of nodes = {}", self.scene_graph.len());
        // println!("total number of edges = {}", total_number_of_edges);

        while dfs_stack.len() > 0 {
            // if dfs_stack.len() > total_number_of_edges {
            //     panic!("ERROR: the scene graph has a cycle");
            // }
            cur_node = dfs_stack.pop().unwrap();
            cur_VM = matrix_stack.pop().unwrap();

            // draw all models at curr_node
            for i in 0..cur_node.models.len() {
                let modelview: glm::TMat4<f32> = cur_VM * (cur_node.models[i].1.transform);
                let curr_model = self.objects_and_instances.get_mut(&cur_node.models[i].0);
                match curr_model{
                    Some(obj) => {
                        // add the Instance to the existing model entry
                        obj.push(Instance{transform: modelview});
                    },
                    None => {
                        // add the new model to the hashmap
                        self.objects_and_instances.insert(cur_node.models[i].0.clone(), vec![Instance{transform: modelview}]);
                    }

                }

                // draw in render() function after creating InstancedModel objects
            }

            for node in cur_node.childnodes.iter() {
                dfs_stack.push(&self.scene_graph.get(node).unwrap().0);
                matrix_stack.push(cur_VM * (self.scene_graph.get(node).unwrap().1.transform));
            }
        }
    }

    pub fn init_scene_graph(&mut self) {
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0
        );

        let mut world_node = Node::new();
        let mut player_node = Node::new();
        let mut island_node = Node::new();
        let mut table_node = Node::new();
        let mut table_leg_node = Node::new();
        let mut table_top_node = Node::new();
        let mut ferris_node = Node::new();
        
        let player_instance_m = Instance{transform: glm::scale( &mat4_identity, &glm::vec3(0.0,0.0,0.0))};
        player_node.models.push((ModelIndex{index: ModelIndices::PLAYER as usize}, player_instance_m));

        let ferris_instance_m = Instance{transform: glm::scale( &mat4_identity, &glm::vec3(1.0,1.0,1.0))};
        ferris_node.models.push((ModelIndex{index: ModelIndices::FERRIS as usize}, ferris_instance_m));

        let table_top_instance_m = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,-0.1,0.0))* glm::scale( &mat4_identity, &glm::vec3(2.0,0.2,1.0))};
        table_top_node.models.push((ModelIndex{index: ModelIndices::CUBE as usize}, table_top_instance_m));
        table_top_node.childnodes.push("ferris".to_string()); 

        let table_leg_instance_m = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,0.5,0.0))* glm::scale( &mat4_identity, &glm::vec3(0.2,1.0,0.2))};
        table_leg_node.models.push((ModelIndex{index: ModelIndices::CUBE as usize}, table_leg_instance_m));

        let table_top_instance_c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,1.5,0.0))};
        table_node.childnodes.push("table top".to_string());
        let table_leg_instance_1c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(-1.7,0.0,-0.7))};
        table_node.childnodes.push("table leg 1".to_string());
        let table_leg_instance_2c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(-1.7,0.0,0.7))};
        table_node.childnodes.push("table leg 2".to_string());
        let table_leg_instance_3c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(1.7,0.0,0.7))};
        table_node.childnodes.push("table leg 3".to_string());
        let table_leg_instance_4c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(1.7,0.0,-0.7))};
        table_node.childnodes.push("table leg 4".to_string());

        island_node.models.push((ModelIndex{index: ModelIndices::ISLAND as usize}, Instance{transform: mat4_identity}));
        let table_instance_c = Instance{transform: glm::rotate(&mat4_identity,-120.0*glm::pi::<f32>()/180.0, &glm::vec3(0.0, 1.0, 0.0)) * glm::translate( &mat4_identity, &glm::vec3(0.0,4.0,0.0))};
        island_node.childnodes.push("table".to_string());
       
        let island_instance_c = Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,-9.7,0.0))};
        world_node.childnodes.push("island".to_string());
        
        world_node.childnodes.push("player".to_string());
        
        // println!("scene graph: {:?}", self.scene_graph);

        self.scene_graph.insert("world".to_string(), (world_node.clone(), Instance{transform: mat4_identity}));
        self.scene_graph.insert("ferris".to_string(), (ferris_node.clone(), Instance{transform: mat4_identity}));
        self.scene_graph.insert("table top".to_string(), (table_top_node.clone(), table_top_instance_c));
        self.scene_graph.insert("table leg 1".to_string(), (table_leg_node.clone(), table_leg_instance_1c));
        self.scene_graph.insert("table leg 2".to_string(), (table_leg_node.clone(), table_leg_instance_2c));
        self.scene_graph.insert("table leg 3".to_string(), (table_leg_node.clone(), table_leg_instance_3c));
        self.scene_graph.insert("table leg 4".to_string(), (table_leg_node.clone(), table_leg_instance_4c));
        self.scene_graph.insert("table".to_string(), (table_node.clone(), table_instance_c));
        self.scene_graph.insert("island".to_string(), (island_node.clone(), island_instance_c));
        self.scene_graph.insert("player".to_string(), (player_node.clone(), Instance{transform: mat4_identity}));
        
    }
}