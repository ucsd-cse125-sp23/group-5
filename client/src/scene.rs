use crate::camera::{Camera, CameraState};
use crate::instance::{Instance, Transform};
use crate::model::{self, InstancedModel, Model};
use crate::{instance, resources};
use glm::{Quat, TMat4, TVec3};
use log::{debug, error, info};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::player::{Player, PlayerController};
use common::core::states::GameState;
use nalgebra_glm as glm;

pub enum ModelIndices {
    ISLAND = 0,
    PLAYER = 1,
    CUBE = 2,
    FERRIS = 3,
}

pub type NodeId = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeInstanceID {
    WORLD_NODE = 0,
    PLAYER_NODE = 1,
    ISLAND_NODE = 2,
    TABLE_NODE = 3,
    TABLE_TOP_NODE = 4,
    TABLE_LEG1_NODE = 5,
    TABLE_LEG2_NODE = 6,
    TABLE_LEG3_NODE = 7,
    TABLE_LEG4_NODE = 8,
    FERRIS_NODE = 9,
}

static OTHER_PLAYER_NODE_ID_START: u32 = 256;
static PREVIOUS_PLAYER_COUNT: u32 = 256;

type ModelIndex = usize;

#[derive(Clone, Debug)]
pub struct Node {
    pub child_ids: Vec<NodeId>,
    pub models: Vec<(ModelIndex, Transform)>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            child_ids: Vec::new(),
            models: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model_index: ModelIndex, transform: Transform) {
        self.models.push((model_index, transform));
    }

    pub fn add_child_id(&mut self, node_id: NodeId) {
        self.child_ids.push(node_id);
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scene {
    pub objects: HashMap<ModelIndex, model::Model>,
    pub scene_graph: HashMap<NodeId, (Node, Transform)>,
    pub objects_and_instances: HashMap<ModelIndex, Vec<Instance>>,
}

pub enum NodeKind {
    Player = 1 << 0,
    Object = 1 << 8,
    World = 1 << 16,
}

impl NodeKind {
    pub fn base_id(&self) -> u32 {
        match self {
            NodeKind::Player => 0,
            NodeKind::Object => 256,
            NodeKind::World => 512,
        }
    }

    pub fn node_id(&self, offset: u8) -> u32 {
        self.base_id() + offset as u32
    }

    pub fn offset_from_base(&self, node_id: u32) -> u8 {
        (node_id - self.base_id()) as u8
    }
}

impl Scene {
    pub fn new(objs: HashMap<ModelIndex, model::Model>) -> Self {
        Scene {
            objects: objs,
            scene_graph: HashMap::from([(NodeKind::World.base_id(), (Node::new(), glm::identity()))]),
            objects_and_instances: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: NodeId, transform: Transform) -> &mut Node {
        let node = Node::new();
        self.scene_graph.insert(node_id, (node, transform));
        let (node, _) = self.scene_graph.get_mut(&node_id).unwrap();
        node
    }

    pub fn add_child_node(&mut self,
                          parent_node_id: NodeId,
                          child_node_id: NodeId,
                          transform: Transform,
    ) -> &mut Node {
        // get the parent node and push the child node to its child ids
        let (parent_node, _) = self.scene_graph.get_mut(&parent_node_id).unwrap();
        parent_node.child_ids.push(child_node_id);

        // add the child node to the scene graph
        self.add_node(child_node_id, transform);
        let (child_node, _) = self.scene_graph.get_mut(&child_node_id).unwrap();
        child_node
    }

    pub fn add_world_child_node(&mut self,
                                child_node_id: NodeId,
                                transform: Transform,
    ) -> &mut Node {
        // get the parent node and push the child node to its child ids
        let (parent_node, _) = self.scene_graph.get_mut(&NodeKind::World.base_id()).unwrap();
        parent_node.child_ids.push(child_node_id);

        // add the child node to the scene graph
        self.add_node(child_node_id, transform);
        let (child_node, _) = self.scene_graph.get_mut(&child_node_id).unwrap();
        child_node
    }

    pub fn load_game_state(
        &mut self,
        game_state: MutexGuard<GameState>,
        player_controller: &mut PlayerController,
        player: &mut Player,
        camera_state: &mut CameraState,
        dt: instant::Duration,
        client_id: u8,
    ) {
        let player_id = client_id as u32; // TODO: why are we using u8 for client_id and u32 for player_id?

        // only render when i'm there
        if game_state.players.contains_key(&player_id) {
            game_state.players.iter().for_each(
                |(id, _player_state)| {
                    let node_id = NodeKind::Player.node_id((*id - 1) as u8);
                    if !self.scene_graph.contains_key(&node_id) {
                        self.add_player_node(node_id);
                    }
                }
            );

            let player_state = &game_state.players.get(&player_id).unwrap();

            player_controller.update(player, camera_state, player_state, dt);

            for (id, player_state) in game_state.players.iter() {
                let node_id = NodeKind::Player.node_id((*id - 1) as u8);
                self.scene_graph.get_mut(&node_id).unwrap().1 = Player::calc_transf_matrix(
                    player_state.transform.translation,
                    player_state.transform.rotation,
                );
            }
        }
    }

    pub fn draw_scene_dfs(&mut self) {
        // get the view matrix from the camera
        self.objects_and_instances.clear();
        let mat4_identity = glm::identity();

        // stacks needed for DFS:
        let mut dfs_stack: Vec<&Node> = Vec::new();
        let mut matrix_stack: Vec<TMat4<f32>> = Vec::new();

        // state needed for DFS:
        let mut cur_node: &Node = &self.scene_graph.get(&NodeKind::World.base_id()).unwrap().0;
        let mut current_view_matrix: TMat4<f32> = mat4_identity;
        dfs_stack.push(cur_node);
        matrix_stack.push(current_view_matrix);

        let mut total_number_of_edges: usize = 0;
        for n in self.scene_graph.iter() {
            total_number_of_edges += n.1.0.child_ids.len();
        }

        debug!("total number of nodes = {}", self.scene_graph.len());
        debug!("total number of edges = {}", total_number_of_edges);

        while !dfs_stack.is_empty() {
            if dfs_stack.len() > total_number_of_edges {
                panic!("ERROR: the scene graph has a cycle");
            }
            cur_node = dfs_stack.pop().unwrap();
            current_view_matrix = matrix_stack.pop().unwrap();

            // draw all models at curr_node
            for i in 0..cur_node.models.len() {
                let model_view: TMat4<f32> = current_view_matrix * (cur_node.models[i].1);
                let curr_model = self.objects_and_instances.get_mut(&cur_node.models[i].0);
                match curr_model {
                    Some(obj) => {
                        // add the Instance to the existing model entry
                        obj.push(Instance {
                            transform: model_view,
                        });
                    }
                    None => {
                        // add the new model to the hashmap
                        self.objects_and_instances.insert(
                            cur_node.models[i].0,
                            vec![Instance {
                                transform: model_view,
                            }],
                        );
                    }
                }
            }

            for node_id in cur_node.child_ids.iter() {
                let (node, transform) = self.scene_graph.get(node_id).unwrap();
                dfs_stack.push(node);
                matrix_stack.push(current_view_matrix * transform);
            }
        }
    }

    pub fn init_scene_graph(&mut self) {
        self.add_player_node(NodeKind::Player.node_id(1));

        self.add_world_child_node(NodeKind::Object.node_id(1),
                                  glm::translate(&glm::identity(), &glm::vec3(0.0, -9.7, 0.0)),
        )
            .add_model(ModelIndices::ISLAND as usize, glm::identity());

        self.add_child_node(NodeKind::Player.node_id(1),
                            NodeKind::Object.node_id(2),
                            glm::translate(&glm::identity(), &glm::vec3(0.0, 1.0, 0.0)),
        )
            .add_model(ModelIndices::FERRIS as usize, glm::identity());
    }

    pub fn add_player_node(&mut self, node_id: NodeId) {
        self.add_world_child_node(node_id,
                                  glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, 0.0)),
        )
            .add_model(ModelIndices::PLAYER as usize, glm::identity());
    }
}
