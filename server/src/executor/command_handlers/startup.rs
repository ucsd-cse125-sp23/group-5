use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::obj_collider::FromObject;
use crate::simulation::physics_state::PhysicsState;
use common::configs::model_config::ConfigModels;
use common::configs::scene_config::ConfigSceneGraph;
use common::core::states::GameState;
use derive_more::Constructor;
use itertools::Itertools;
use nalgebra::{Isometry3, UnitQuaternion};
use rapier3d::math::Isometry;
use rapier3d::{dynamics, geometry};

#[derive(Constructor)]
/// Handles the startup command that initializes the games state and physics world
pub struct StartupCommandHandler {
    config_models: ConfigModels,
    config_scene_graph: ConfigSceneGraph,
}

impl CommandHandler for StartupCommandHandler {
    fn handle(
        &self,
        _: &mut GameState,
        physics_state: &mut PhysicsState,
        _game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let mut scene_entity_id = 0xBEEF; // TODO: set up a better convention for scene entities

        let mut nodes = self
            .config_scene_graph
            .nodes
            .iter()
            .map(|n| (n.clone(), Isometry::identity()))
            .collect_vec();

        while !nodes.is_empty() {
            let (node, parent_transform) = nodes.pop().unwrap();
            let model = node.model.clone().ok_or(HandlerError::new(
                "Config node not attaching model".to_string(),
            ))?;

            let model_config = self
                .config_models
                .model(model)
                .ok_or(HandlerError::new("Model not declared".to_string()))?;

            let (models, _) = tobj::load_obj(model_config.path.clone(), &tobj::GPU_LOAD_OPTIONS)
                .map_err(|e| HandlerError::new(format!("Error loading model {:?}", e)))?;

            let local_transform = Isometry3::from_parts(
                node.transform.position.into(),
                UnitQuaternion::from_quaternion(node.transform.rotation),
            );

            let world_transform = parent_transform * local_transform;

            let body = dynamics::RigidBodyBuilder::fixed()
                .position(world_transform)
                .build();

            let decompose = node.decompose.unwrap_or(false);

            let collider = if !model_config.phantom.unwrap_or(false) {
                Some(geometry::ColliderBuilder::from_object_models(models, decompose).build())
            } else {
                None
            };

            physics_state.insert_entity(scene_entity_id, collider, Some(body)); // insert the collider into the physics world
            scene_entity_id += 1;

            // add children to nodes
            if let Some(children) = node.children.clone() {
                nodes.extend(
                    children
                        .iter()
                        .map(|child| (child.clone(), world_transform)),
                );
            }
        }

        Ok(())
    }
}
