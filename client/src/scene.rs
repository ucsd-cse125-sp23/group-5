use crate::camera::{Camera, CameraState};
use crate::instance::{Instance, Transform};
use crate::model::{self, InstancedModel, Model};
use glm::{identity, Mat4, Quat, TMat3, TMat4, TVec3};
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::{MutexGuard};

use crate::player::{Player, PlayerController};
use common::core::states::GameState;
use nalgebra_glm as glm;
use common::configs::scene_config::{ConfigNode, ConfigSceneGraph};
use common::configs::model_config::ModelIndex;


pub enum ModelIndices {
    ISLAND = 0,
    PLAYER = 1,
    CUBE = 2,
    FERRIS = 3,
}

pub type NodeId = String;


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

    pub fn add_model(&mut self, model_index: ModelIndex) {
        self.models.push((model_index, glm::identity()));
    }

    pub fn add_model_at(&mut self, model_index: ModelIndex, transform: Transform, index: usize) {
        self.models.insert(index, (model_index, transform));
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scene {
    pub objects: HashMap<ModelIndex, model::Model>,
    pub scene_graph: HashMap<NodeId, (Node, Transform)>,
    pub objects_and_instances: HashMap<ModelIndex, Vec<Instance>>,
}

pub enum NodeKind {
    Player,
    Object,
    World,
}

impl NodeKind {
    pub fn base_id(&self) -> NodeId {
        match self {
            NodeKind::Player => "player",
            NodeKind::Object => "object",
            NodeKind::World => "world"
        }.to_string()
    }

    // anything that can be displayed
    pub fn node_id(&self, tag: impl Into<NodeId>) -> NodeId {
        format!("{}:{}", self.base_id(), tag.into())
    }

    pub fn from_node_id(id: &NodeId) -> Option<Self> {
        match id.split(":").next().unwrap() {
            "player" => Some(NodeKind::Player),
            "object" => Some(NodeKind::Object),
            "world" => Some(NodeKind::World),
            _ => None
        }
    }
}

impl Scene {
    pub fn new(objs: HashMap<ModelIndex, Model>) -> Self {
        Scene {
            objects: objs,
            scene_graph: HashMap::from([(NodeKind::World.base_id(), (Node::new(), glm::identity()))]),
            objects_and_instances: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: NodeId, transform: Transform) -> &mut Node {
        let node = Node::new();
        self.scene_graph.insert(node_id.clone(), (node, transform));
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
        parent_node.child_ids.push(child_node_id.clone());

        // add the child node to the scene graph
        self.add_node(child_node_id.clone(), transform);
        let (child_node, _) = self.scene_graph.get_mut(&child_node_id).unwrap();
        child_node
    }

    pub fn add_world_child_node(&mut self,
                                child_node_id: NodeId,
                                transform: Transform,
    ) -> &mut Node {
        // get the parent node and push the child node to its child ids
        let (parent_node, _) = self.scene_graph.get_mut(&NodeKind::World.base_id()).unwrap();
        parent_node.child_ids.push(child_node_id.clone());

        // add the child node to the scene graph
        self.add_node(child_node_id.clone(), transform);
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
                    let node_id = NodeKind::Player.node_id(id.to_string());
                    if !self.scene_graph.contains_key(&node_id) {
                        self.add_player_node(node_id);
                    }
                }
            );

            let player_state = &game_state.players.get(&player_id).unwrap();

            player_controller.update(player, camera_state, player_state, dt);

            for (id, player_state) in game_state.players.iter() {
                let node_id = NodeKind::Player.node_id(id.to_string());
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
                            cur_node.models[i].0.clone(),
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
        // self.add_player_node(NodeKind::Player.base_id());
        //
        // self.add_world_child_node(NodeKind::Object.node_id("island"),
        //                           glm::translate(&glm::identity(), &glm::vec3(0.0, -9.7, 0.0)),
        // )
        //     .add_model('island'.to_string());
        //
        // self.add_child_node(NodeKind::Player.base_id(),
        //                     NodeKind::Object.node_id("ferris"),
        //                     glm::translate(&glm::identity(), &glm::vec3(0.0, 1.0, 0.0)),
        // )
        //     .add_model(ModelIndices::FERRIS as usize);
    }

    pub fn add_player_node(&mut self, node_id: NodeId) {
        self.add_world_child_node(node_id,
                                  glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, 0.0)),
        )
            .add_model("player".to_string());
    }
}


impl Scene {
    pub fn from_config(
        json_scene_graph: &ConfigSceneGraph,
    ) -> Self {
        let mut scene = Self::new(HashMap::new());

        for node in &json_scene_graph.nodes {
            Scene::add_node_from_config(&mut scene, node, None);
        }
        scene
    }

    fn add_node_from_config(
        scene: &mut Scene,
        json_node: &ConfigNode,
        parent_id: Option<NodeId>,
    ) {
        let node_transform = Transform::new_translation(&json_node.transform.position)
            * glm::quat_to_mat4(&json_node.transform.rotation);

        let node = match parent_id {
            Some(parent_id) => scene.add_child_node(parent_id, json_node.id.clone(), node_transform),
            None => scene.add_world_child_node(json_node.id.clone(), node_transform),
        };

        if let Some(model_index) = json_node.model.clone() {
            node.add_model(model_index);
        }

        if let Some(ref children) = json_node.children {
            for child in children {
                Scene::add_node_from_config(scene, child, Some(json_node.id.clone()));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use common::configs::scene_config::ConfigSceneGraph;
    use super::Scene;

    // Test reading a scene graph from an inline JSON string
    #[test]
    fn test_from_json_scene_graph() {
        // Define the JSON string
        let json_scene_graph_str = r#"
        {
          "nodes": [
            {
              "id": "object:island",
              "model": "island",
              "transform": {
                "position": [0, -9.7, 0],
                "rotation": [0, 0, 0, 1]
              },
              "children": [
                {
                  "id": "object:ferris",
                  "transform": {
                    "position": [0, 5, 0],
                    "rotation": [0, 0.2, 0, 1]
                  },
                  "model": "ferris"
                }
              ]
            }
          ]
        }
        "#;

        // Deserialize the JSON string into a ConfigSceneGraph
        let json_scene_graph: ConfigSceneGraph = serde_json::from_str(json_scene_graph_str).unwrap();

        // Create a Scene from the ConfigSceneGraph
        let scene = Scene::from_config(&json_scene_graph);

        // Test if the scene was created successfully
        assert!(scene.scene_graph.contains_key("object:island"));
        assert!(scene.scene_graph.contains_key("object:ferris"));
    }
}