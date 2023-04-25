use crate::camera::{Camera, CameraState};
use crate::instance::Instance;
use crate::model::{self, InstancedModel, Model};
use crate::{instance, resources};
use glm::{Quat, TMat4, TVec3};
use log::error;
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeID {
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

static OTHER_PLAYER_NODE_ID_START: AtomicI32 = AtomicI32::new(-1);
static PREVIOUS_PLAYER_COUNT: AtomicI32 = AtomicI32::new(1);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelIndex {
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub childnodes: Vec<i32>,
    pub models: Vec<(ModelIndex, Instance)>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            childnodes: Vec::new(),
            models: Vec::new(),
        }
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scene {
    pub objects: Vec<model::Model>,
    pub scene_graph: HashMap<i32, (Node, Instance)>,
    pub objects_and_instances: HashMap<ModelIndex, Vec<Instance>>,
    pub index_to_id_map: HashMap<usize, u32>,
}

impl Scene {
    pub fn new(objs: Vec<model::Model>) -> Self {
        Scene {
            objects: objs,
            scene_graph: HashMap::new(),
            objects_and_instances: HashMap::new(),
            index_to_id_map: HashMap::new(),
        }
    }

    pub fn load_game_state(
        &mut self,
        game_state: MutexGuard<GameState>,
        player_controller: &mut PlayerController,
        player: &mut Player,
        camera: &mut Camera,
        dt: instant::Duration,
        client_id: u8,
    ) {
        // update the camera target
        if game_state.players.contains_key(&(client_id as u32)) {
            // when a new player come in
            if PREVIOUS_PLAYER_COUNT.fetch_add(0, Ordering::SeqCst) as usize
                != game_state.players.len()
            {
                self.add_other_player();
                self.draw_scene_dfs(camera);
                PREVIOUS_PLAYER_COUNT.fetch_add(1, Ordering::SeqCst);
                let mut new_mapped_id: u32 = 1;
                while !(game_state.players.contains_key(&new_mapped_id)
                    && !self.index_to_id_map.values().any(|&x| x == new_mapped_id))
                {
                    new_mapped_id = new_mapped_id + 1;
                }
                self.index_to_id_map
                    .insert(self.index_to_id_map.len() as usize, new_mapped_id);
            }
            let player_index = (client_id) as usize;
            let player_state = &game_state.players.get(&(player_index as u32)).unwrap();
            if player_index != (player_state.id) as usize {
                error!("ids don't match");
            }
            // update player controller (player, camera, etc) with the latest player state
            player_controller.update(player, camera, player_state, dt);
            // according to each player's instance, rendering their object
            let player_instances = self
                .objects_and_instances
                .get_mut(&ModelIndex {
                    index: ModelIndices::PLAYER as usize,
                })
                .unwrap();
            for (index, client_player_instance) in player_instances.iter_mut().enumerate() {
                let client_player_state = game_state
                    .players
                    .get(self.index_to_id_map.get(&index).unwrap())
                    .unwrap();
                client_player_instance.transform = Player::calc_transf_matrix(
                    client_player_state.transform.translation,
                    client_player_state.transform.rotation,
                );
            }
        } else {
            self.index_to_id_map.insert(0, client_id as u32);
        }
    }

    pub fn draw_scene_dfs(&mut self, camera: &Camera) {
        // get the view matrix from the camera
        self.objects_and_instances.clear();
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        // stacks needed for DFS:
        let mut dfs_stack: Vec<&Node> = Vec::new();
        let mut matrix_stack: Vec<glm::TMat4<f32>> = Vec::new();

        // state needed for DFS:
        // self.scene_graph.get("world").unwrap(); // should be the root of the tree to start --> "world"
        let mut cur_node: &Node = &self
            .scene_graph
            .get(&(NodeID::WORLD_NODE as i32))
            .unwrap()
            .0;
        // camera.calc_matrix(); // should be the camera's view matrix to start --> "world"s modelview matrix is the camera's view matrix
        // currently it's just the identity matrix!
        let mut cur_VM: glm::TMat4<f32> = mat4_identity;

        dfs_stack.push(cur_node);
        matrix_stack.push(cur_VM);

        let mut total_number_of_edges: usize = 0;
        for n in self.scene_graph.iter() {
            total_number_of_edges += n.1 .0.childnodes.len();
        }

        println!("total number of nodes = {}", self.scene_graph.len());
        println!("total number of edges = {}", total_number_of_edges);

        while !dfs_stack.is_empty() {
            if dfs_stack.len() > total_number_of_edges {
                panic!("ERROR: the scene graph has a cycle");
            }
            cur_node = dfs_stack.pop().unwrap();
            cur_VM = matrix_stack.pop().unwrap();

            // draw all models at curr_node
            for i in 0..cur_node.models.len() {
                let modelview: glm::TMat4<f32> = cur_VM * (cur_node.models[i].1.transform);
                let curr_model = self.objects_and_instances.get_mut(&cur_node.models[i].0);
                match curr_model {
                    Some(obj) => {
                        // add the Instance to the existing model entry
                        obj.push(Instance {
                            transform: modelview,
                        });
                    }
                    None => {
                        // add the new model to the hashmap
                        self.objects_and_instances.insert(
                            cur_node.models[i].0.clone(),
                            vec![Instance {
                                transform: modelview,
                            }],
                        );
                    }
                }

                // draw in render() function after creating InstancedModel objects
            }
            for node in cur_node.childnodes.iter() {
                let node_id = node.clone() as i32;
                dfs_stack.push(&self.scene_graph.get(&node_id).unwrap().0);
                matrix_stack.push(cur_VM * (self.scene_graph.get(&node_id).unwrap().1.transform));
            }
        }
    }

    pub fn init_scene_graph(&mut self) {
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        let mut world_node = Node::new();
        let mut player_node = Node::new();
        let mut island_node = Node::new();
        let mut table_node = Node::new();
        let mut table_leg_node = Node::new();
        let mut table_top_node = Node::new();
        let mut ferris_node = Node::new();

        let player_instance_m = Instance {
            transform: glm::scale(&mat4_identity, &glm::vec3(0.0, 0.0, 0.0)),
        };
        player_node.models.push((
            ModelIndex {
                index: ModelIndices::PLAYER as usize,
            },
            player_instance_m,
        ));

        let ferris_instance_m = Instance {
            transform: glm::scale(&mat4_identity, &glm::vec3(1.0, 1.0, 1.0)),
        };
        ferris_node.models.push((
            ModelIndex {
                index: ModelIndices::FERRIS as usize,
            },
            ferris_instance_m,
        ));

        let table_top_instance_m = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(0.0, -0.1, 0.0))
                * glm::scale(&mat4_identity, &glm::vec3(2.0, 0.2, 1.0)),
        };
        table_top_node.models.push((
            ModelIndex {
                index: ModelIndices::CUBE as usize,
            },
            table_top_instance_m,
        ));
        table_top_node.childnodes.push(NodeID::FERRIS_NODE as i32);

        let table_leg_instance_m = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(0.0, 0.5, 0.0))
                * glm::scale(&mat4_identity, &glm::vec3(0.2, 1.0, 0.2)),
        };
        table_leg_node.models.push((
            ModelIndex {
                index: ModelIndices::CUBE as usize,
            },
            table_leg_instance_m,
        ));

        let table_top_instance_c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(0.0, 1.5, 0.0)),
        };
        table_node.childnodes.push(NodeID::TABLE_TOP_NODE as i32);
        let table_leg_instance_1c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(-1.7, 0.0, -0.7)),
        };
        table_node.childnodes.push(NodeID::TABLE_LEG1_NODE as i32);
        let table_leg_instance_2c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(-1.7, 0.0, 0.7)),
        };
        table_node.childnodes.push(NodeID::TABLE_LEG2_NODE as i32);
        let table_leg_instance_3c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(1.7, 0.0, 0.7)),
        };
        table_node.childnodes.push(NodeID::TABLE_LEG3_NODE as i32);
        let table_leg_instance_4c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(1.7, 0.0, -0.7)),
        };
        table_node.childnodes.push(NodeID::TABLE_LEG4_NODE as i32);

        island_node.models.push((
            ModelIndex {
                index: ModelIndices::ISLAND as usize,
            },
            Instance {
                transform: mat4_identity,
            },
        ));
        let table_instance_c = Instance {
            transform: glm::rotate(
                &mat4_identity,
                -120.0 * glm::pi::<f32>() / 180.0,
                &glm::vec3(0.0, 1.0, 0.0),
            ) * glm::translate(&mat4_identity, &glm::vec3(0.0, 4.0, 0.0)),
        };
        island_node.childnodes.push(NodeID::TABLE_NODE as i32);

        let island_instance_c = Instance {
            transform: glm::translate(&mat4_identity, &glm::vec3(0.0, -9.7, 0.0)),
        };
        world_node.childnodes.push(NodeID::ISLAND_NODE as i32);

        world_node.childnodes.push(NodeID::PLAYER_NODE as i32);

        // println!("scene graph: {:?}", self.scene_graph);

        self.scene_graph.insert(
            NodeID::WORLD_NODE as i32,
            (
                world_node.clone(),
                Instance {
                    transform: mat4_identity,
                },
            ),
        );
        self.scene_graph.insert(
            NodeID::FERRIS_NODE as i32,
            (
                ferris_node.clone(),
                Instance {
                    transform: mat4_identity,
                },
            ),
        );
        self.scene_graph.insert(
            NodeID::TABLE_TOP_NODE as i32,
            (table_top_node.clone(), table_top_instance_c),
        );
        self.scene_graph.insert(
            NodeID::TABLE_LEG1_NODE as i32,
            (table_leg_node.clone(), table_leg_instance_1c),
        );
        self.scene_graph.insert(
            NodeID::TABLE_LEG2_NODE as i32,
            (table_leg_node.clone(), table_leg_instance_2c),
        );
        self.scene_graph.insert(
            NodeID::TABLE_LEG3_NODE as i32,
            (table_leg_node.clone(), table_leg_instance_3c),
        );
        self.scene_graph.insert(
            NodeID::TABLE_LEG4_NODE as i32,
            (table_leg_node.clone(), table_leg_instance_4c),
        );
        self.scene_graph.insert(
            NodeID::TABLE_NODE as i32,
            (table_node.clone(), table_instance_c),
        );
        self.scene_graph.insert(
            NodeID::ISLAND_NODE as i32,
            (island_node.clone(), island_instance_c),
        );
        self.scene_graph.insert(
            NodeID::PLAYER_NODE as i32,
            (
                player_node.clone(),
                Instance {
                    transform: mat4_identity,
                },
            ),
        );
    }

    pub fn add_other_player(&mut self) {
        let mat4_identity = glm::mat4(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        let mut world_node = Node::new();
        let mut other_player_node = Node::new();

        let other_player_instance_m = Instance {
            transform: glm::scale(&mat4_identity, &glm::vec3(0.0, 0.0, 0.0)),
        };
        other_player_node.models.push((
            ModelIndex {
                index: ModelIndices::PLAYER as usize,
            },
            other_player_instance_m,
        ));

        let other_player_node_id: i32 = OTHER_PLAYER_NODE_ID_START.fetch_sub(1, Ordering::SeqCst);

        let mut world_childnodes = self
            .scene_graph
            .get(&(NodeID::WORLD_NODE as i32))
            .unwrap()
            .0
            .childnodes
            .clone();

        world_childnodes.push(other_player_node_id as i32);
        world_node.childnodes = world_childnodes.clone();

        self.scene_graph.remove(&(NodeID::WORLD_NODE as i32));

        self.scene_graph.insert(
            NodeID::WORLD_NODE as i32,
            (
                world_node.clone(),
                Instance {
                    transform: mat4_identity,
                },
            ),
        );

        self.scene_graph.insert(
            other_player_node_id as i32,
            (
                other_player_node,
                Instance {
                    transform: mat4_identity,
                },
            ),
        );
    }
}
