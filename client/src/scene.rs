use crate::camera::Camera;
use crate::{instance, resources};
use crate::instance::Instance;
use crate::model::{self, InstancedModel, Model};
use std::collections::HashMap;
use std::cell::Cell;

use nalgebra_glm as glm;

enum ModelIndices{
    ISLAND = 0,
    CUBE = 1,
    FERRIS = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelIndex{
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Node {
    // pub parent: Box<Node>,
    pub childnodes: Vec<Node>,
    pub childtransforms: Vec<Instance>,
    pub models: Vec<ModelIndex>,
    pub modeltransforms: Vec<Instance>,
}

impl Node{
    pub fn new() -> Self {
        Node {
            // parent: Box::new(Node::new()),
            childnodes: Vec::new(),
            childtransforms: Vec::new(),
            models: Vec::new(),
            modeltransforms: Vec::new(),
        }
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scene{
    pub objects: Vec<model::Model>,
    // pub instance_vectors : Vec<Vec<Instance>>,

    // pub root: Node,
    // pub scene_graph: HashMap<String, Node>,
    pub scene_graph: Vec<Node>, // make sure that the root node is at index 0
    pub objects_and_instances: HashMap<ModelIndex, Vec<Instance>>,
}

impl Scene{
    pub fn new(objs: Vec<model::Model>) -> Self {
        Scene {
            objects: objs,
            // instance_vectors: Vec::new(),

            //root: Node::new(),
            //scene_graph: HashMap::new(),
            scene_graph: Vec::new(),
            objects_and_instances: HashMap::new(),
        }
    }

    pub fn draw_scene_dfs(&mut self, camera: &Camera){ // get the view matrix from the camera
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
        let mut cur_node: &Node = &self.scene_graph[0]; // self.scene_graph.get("world").unwrap(); // should be the root of the tree to start --> "world"
        let mut cur_VM: glm::TMat4<f32> = mat4_identity; //camera.calc_matrix(); // should be the camera's view matrix to start --> "world"s modelview matrix is the camera's view matrix
                                                                    // currently it's just the identity matrix!
        dfs_stack.push(cur_node);
        matrix_stack.push(cur_VM);

        let mut total_number_of_edges: usize = 0;
        for n in self.scene_graph.iter() {
           total_number_of_edges += n.childnodes.len();
        }

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
                let modelview: glm::TMat4<f32> = cur_VM * (cur_node.modeltransforms[i].transform);
                let curr_model = self.objects_and_instances.get_mut(&cur_node.models[i]);
                match curr_model{
                    Some(obj) => {
                        // add the Instance to the existing model entry
                        obj.push(Instance{transform: modelview});
                    },
                    None => {
                        // add the new model to the hashmap
                        self.objects_and_instances.insert(cur_node.models[i].clone(), vec![Instance{transform: modelview}]);
                    }

                }

                // draw in render() function after creating InstancedModel objects
            }

            for i in 0..cur_node.childnodes.len() {
                dfs_stack.push(&cur_node.childnodes[i]);
                matrix_stack.push(cur_VM * (cur_node.childtransforms[i].transform));
            }
        }
    }

    pub fn build_scene_graph(&mut self) {
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0
        );

        let mut island_node = Node::new();
        let mut table_node = Node::new();
        let mut table_top_node = Node::new();
        let mut table_leg_node = Node::new();
        let mut world_node = Node::new();
        let mut ferris_node = Node::new();


        // TODO: MAKE SCENE GRAPH DYNAMIC
        // let mut player_node = Node::new();
        // player_node.models.push(ModelIndex{index: ModelIndices::PLAYER as usize});
        // let player_instance: Instance = FUNCTION_THAT_RETURNS_PLAYER_TRANSFORM() or pass it in as an argument;
        // player_node.modeltransforms.push(player_instance);
        // player nodes are children of world nodes --> lines 179-180

        ferris_node.models.push(ModelIndex{index: ModelIndices::FERRIS as usize});
        ferris_node.modeltransforms.push(Instance{transform: glm::scale( &mat4_identity, &glm::vec3(1.0,1.0,1.0))});

        table_top_node.models.push(ModelIndex{index: ModelIndices::CUBE as usize});
        table_top_node.modeltransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,-0.1,0.0))* glm::scale( &mat4_identity, &glm::vec3(2.0,0.2,1.0))});
        table_top_node.childnodes.push(ferris_node.clone());
        table_top_node.childtransforms.push(Instance{transform: mat4_identity}); //glm::translate( &mat4_identity, &glm::vec3(1.7,0.0,-0.7))});

        table_leg_node.models.push(ModelIndex{index: ModelIndices::CUBE as usize});
        table_leg_node.modeltransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,0.5,0.0))* glm::scale( &mat4_identity, &glm::vec3(0.2,1.0,0.2))});

        table_node.childnodes.push(table_top_node.clone());
        table_node.childtransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,1.5,0.0))});
        table_node.childnodes.push(table_leg_node.clone());
        table_node.childtransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(-1.7,0.0,-0.7))});
        table_node.childnodes.push(table_leg_node.clone());
        table_node.childtransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(-1.7,0.0,0.7))});
        table_node.childnodes.push(table_leg_node.clone());
        table_node.childtransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(1.7,0.0,0.7))});
        table_node.childnodes.push(table_leg_node.clone());
        table_node.childtransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(1.7,0.0,-0.7))});

        island_node.childnodes.push(table_node.clone());
        island_node.childtransforms.push(Instance{transform: glm::rotate(&mat4_identity,-120.0*glm::pi::<f32>()/180.0, &glm::vec3(0.0, 1.0, 0.0) ) * glm::translate( &mat4_identity, &glm::vec3(2.0,0.0,0.0)) * glm::scale( &mat4_identity, &glm::vec3(1.5,1.5,1.5))});
        island_node.models.push(ModelIndex{index: ModelIndices::ISLAND as usize});
        island_node.modeltransforms.push(Instance{transform: glm::translate( &mat4_identity, &glm::vec3(0.0,-9.7,0.0)) * glm::scale( &mat4_identity, &glm::vec3(2.5,2.5,2.5))});

        world_node.childnodes.push(island_node);
        world_node.childtransforms.push(Instance{transform: mat4_identity});
        
        // world_node.childnodes.push(player_node);
        // world_node.childtransforms.push(Instance{transform: mat4_identity});
        
        self.scene_graph.push(world_node);// self.scene_graph.push(cube_node.clone());

        // println!("scene graph: {:?}", self.scene_graph);
        
    }
}