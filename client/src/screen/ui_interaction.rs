use crate::inputs::ClientSync::Ready;
use crate::inputs::Input;
use crate::screen;
use log::warn;
use phf::phf_map;
use crate::mesh_color::MeshColor;
use crate::screen::FinalChoices;
use crate::screen::Screen;

use super::location_helper::get_coords;
use super::objects::{Button, Icon};

pub static BUTTON_MAP: phf::Map<&'static str, fn(&mut screen::Display, Option<MeshColor>, Option<String>)> = phf_map!{
    "game_start" => game_start,
    "change_leaf_type" => change_leaf_type,
    "change_leaf_color" => change_leaf_color,
    "change_wood_color" => change_wood_color,
    "go_to_lobby" => go_to_lobby,
};

// Place click events here ----------------------
fn game_start(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    if display.customization_choices.0.color.len() >= 2 && !display.customization_choices.1 {
        let curr_group = display.groups.get_mut(&display.current).unwrap();
        let curr_screen = display.screen_map.get_mut(&curr_group.screen.clone().unwrap()).unwrap();
        curr_screen.buttons[*curr_screen.btn_id_map.get("start_game").unwrap()].default_tint = nalgebra_glm::Vec4::new(0.0,1.0,0.0,1.0);
        display.customization_choices.1 = true;
        let final_choices = display.customization_choices.0.clone();
        println!("{:#?}", final_choices);
        // TODO: sened final customization choices to server

        // once start game, send ready to the client main.
        match display.sender.send(Input::UI(Ready)) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error sending command: {:?}", e);
            }
        }

        // display.change_to(display.game_display.clone());
    }
}

fn go_to_lobby(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    display.change_to("display:lobby".to_owned());

    // TODO: reset everything in the lobby (reset color, or keep remember choices?)
    display.customization_choices.1 = false;
    match display.scene_map.get_mut("scene:lobby"){
        None => {},
        Some(scene) => {
            match scene.scene_graph.get_mut("object:player_model") {
                None => {},
                Some(node) => {
                    node.colors = Some(std::collections::HashMap::from([("korok".to_string(), MeshColor::new([0.5,0.5,0.5]))]));
                }
            }
            scene.draw_scene_dfs();
        }
    }
}

fn change_leaf_type(display: &mut screen::Display, _: Option<MeshColor>, button_id: Option<String>){
    if display.customization_choices.1 { return }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display.screen_map.get_mut(&curr_group.screen.clone().unwrap()).unwrap();

    let curr_leaf_type: &mut Icon = &mut curr_screen.icons[*curr_screen.icon_id_map.get("leaf_type_selector").unwrap()];
    let curr_button: &Button = &curr_screen.buttons[*curr_screen.btn_id_map.get(&button_id.clone().unwrap()).unwrap()];

    match curr_group.scene.clone() {
        None => {},
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id){
                None => {},
                Some(scene) => {
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {},
                        Some(node) => {
                            node.model = Some(button_id.clone().unwrap());
                            curr_leaf_type.location = curr_button.location.clone();
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
}

fn change_leaf_color(display: &mut screen::Display, color: Option<MeshColor>, button_id: Option<String>){
    if display.customization_choices.1 { return }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display.screen_map.get_mut(&curr_group.screen.clone().unwrap()).unwrap();

    let curr_leaf_color: &mut Icon = &mut curr_screen.icons[*curr_screen.icon_id_map.get("leaf_color_selector").unwrap()];
    let curr_button: &Button = &curr_screen.buttons[*curr_screen.btn_id_map.get(&button_id.clone().unwrap()).unwrap()];

    let actual_color = match color {None => MeshColor::default(), Some(c) => c};

    match curr_group.scene.clone() {
        None => {},
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id){
                None => {},
                Some(scene) => {
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {},
                        Some(node) => {
                            display.customization_choices.0.color.insert("eyes_eyes_mesh".to_owned(), actual_color);
                            display.customization_choices.0.color.insert("korok".to_owned(), actual_color);
                            node.colors = Some(display.customization_choices.0.color.clone());
                            curr_leaf_color.location = curr_button.location;
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
    if display.customization_choices.0.color.len() >= 2 {
        curr_screen.buttons[*curr_screen.btn_id_map.get("start_game").unwrap()].default_tint = nalgebra_glm::Vec4::new(1.0,1.0,1.0,1.0);
        curr_screen.buttons[*curr_screen.btn_id_map.get("start_game").unwrap()].hover_tint = nalgebra_glm::Vec4::new(0.0,1.0,0.0,1.0);
    }
}

fn change_wood_color(display: &mut screen::Display, color: Option<MeshColor>, button_id: Option<String>){
    if display.customization_choices.1 { return }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display.screen_map.get_mut(&curr_group.screen.clone().unwrap()).unwrap();

    let curr_wood_color: &mut Icon = &mut curr_screen.icons[*curr_screen.icon_id_map.get("wood_color_selector").unwrap()];
    let curr_button: &Button = &curr_screen.buttons[*curr_screen.btn_id_map.get(&button_id.clone().unwrap()).unwrap()];

    let actual_color = match color {None => MeshColor::default(), Some(c) => c};

    match curr_group.scene.clone() {
        None => {},
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id){
                None => {},
                Some(scene) => {
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {},
                        Some(node) => {
                            display.customization_choices.0.color.insert("leg0R_leg0R_mesh".to_owned(), actual_color);
                            node.colors = Some(display.customization_choices.0.color.clone());
                            curr_wood_color.location = curr_button.location;
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
    if display.customization_choices.0.color.len() >= 2 {
        curr_screen.buttons[*curr_screen.btn_id_map.get("start_game").unwrap()].default_tint = nalgebra_glm::Vec4::new(1.0,1.0,1.0,1.0);
        curr_screen.buttons[*curr_screen.btn_id_map.get("start_game").unwrap()].hover_tint = nalgebra_glm::Vec4::new(0.0,1.0,0.0,1.0);
    }
}
// end click events ----------------------