use crate::inputs::ClientSync::Ready;
use crate::inputs::{ClientSync, Input};
use crate::screen;
use crate::screen::{Display, Screen};
use common::core::choices::FinalChoices;
use common::core::mesh_color::MeshColor;
use common::core::states::GameLifeCycleState::Waiting;
use log::warn;
use phf::phf_map;
use std::thread;
use std::time::Duration;

const OBJECT_PLAYER_MODEL: &str = "object:player_model";
const EYES_EYES_MESH: &str = "eyes_eyes_mesh";
const LEG0R_LEG0R_MESH: &str = "leg0R_leg0R_mesh";

pub static BUTTON_MAP: phf::Map<
    &'static str,
    fn(&mut screen::Display, Option<MeshColor>, Option<String>),
> = phf_map! {
    "next_model" => next_model,
    "change_player_color" => change_player_color,
    "customize_body" => customize_body,
    "customize_leaf" => customize_leaf,
    "go_to_lobby" => go_to_lobby,
    "go_to_title" => go_to_title,
    "game_start" => game_start,
};

/// ---------------------------------- Place click events here -------------------------------------
fn game_start(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>) {
    let final_choices = FinalChoices::new(&display.customization_choices);
    // println!("{:#?}", final_choices);
    // Send final customization choices to server
    match display
        .sender
        .send(Input::UI(ClientSync::Choices(final_choices)))
    {
        Ok(_) => {}
        Err(e) => {
            warn!("Error sending command: {:?}", e);
        }
    }
    // once start game, send ready to the client main.
    match display.sender.send(Input::UI(Ready)) {
        Ok(_) => {}
        Err(e) => {
            warn!("Error sending command: {:?}", e);
        }
    }
    //display.change_to(display.game_display.clone());
}

fn customize_body(display: &mut screen::Display, _: Option<MeshColor>, button_id: Option<String>) {
    update_curr_selection(display, button_id, false);
    update_curr_selection(
        display,
        Some(display.customization_choices.cur_body_color.clone()),
        true,
    );
    display.customization_choices.current_type_choice = "body".to_owned();
}

fn customize_leaf(display: &mut screen::Display, _: Option<MeshColor>, button_id: Option<String>) {
    update_curr_selection(display, button_id, false);
    update_curr_selection(
        display,
        Some(display.customization_choices.cur_leaf_color.clone()),
        true,
    );
    display.customization_choices.current_type_choice = "leaf".to_owned();
}

fn change_player_color(
    display: &mut screen::Display,
    color: Option<MeshColor>,
    button_id: Option<String>,
) {
    update_curr_selection(display, button_id.clone(), true);

    let actual_color = color.unwrap_or(MeshColor::default());

    if let Some(curr_group) = display.groups.get_mut(&display.current) {
        if let Some(scene_id) = curr_group.scene.clone() {
            if let Some(scene) = display.scene_map.get_mut(&scene_id) {
                if let Some(node) = scene.scene_graph.get_mut(OBJECT_PLAYER_MODEL) {
                    let curr_choice = display.customization_choices.current_type_choice.clone();
                    if curr_choice == "leaf".to_owned() {
                        display.customization_choices.cur_leaf_color = button_id.clone().unwrap();
                        display
                            .customization_choices
                            .color
                            .insert(EYES_EYES_MESH.to_owned(), actual_color);
                    } else if curr_choice == "body".to_owned() {
                        display.customization_choices.cur_body_color = button_id.clone().unwrap();
                        display
                            .customization_choices
                            .color
                            .insert(LEG0R_LEG0R_MESH.to_owned(), actual_color);
                    }
                    node.colors = Some(display.customization_choices.color.clone());
                }
                scene.draw_scene_dfs();
            }
        }
    }
}

fn go_to_lobby(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    display.change_to("display:lobby".to_owned());
}

fn go_to_title(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    display.current = "display:title".to_owned();
}

fn next_model(display: &mut screen::Display, _: Option<MeshColor>, _: Option<String>){
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    match curr_group.scene.clone() {
        None => {}
        Some(scene_id) => {
            match display.scene_map.get_mut(&scene_id) {
                None => {}
                Some(scene) => {
                    match scene.scene_graph.get_mut("object:player_model") {
                        None => {}
                        Some(node) => {
                            let str = node.model.as_deref();
                            match str {
                                Some(model) => match model {
                                    // TODO: fix later, hard-coded for now
                                    "cube" => {
                                        display.customization_choices.current_model =
                                            "ferris".to_owned();
                                        node.model = Some("ferris".to_string());
                                    }
                                    "ferris" => {
                                        display.customization_choices.current_model =
                                            "cube".to_owned();
                                        node.model = Some("cube".to_string());
                                    }
                                    _ => {}
                                },
                                None => {}
                            }
                        }
                    }
                    scene.draw_scene_dfs();
                }
            }
        }
    }
}
/// -------------------------------------- end click events ----------------------------------------

fn update_curr_selection(
    display: &mut screen::Display,
    button_id: Option<String>,
    color_or_type: bool,
) {
    let col_or_type = if color_or_type {
        &display.customization_choices.prev_color_selection
    } else {
        &display.customization_choices.prev_type_selection
    };

    let display_group = display.groups.get_mut(&display.current).unwrap();
    let screen: &mut Screen;
    match display_group.screen.as_ref() {
        None => return,
        Some(s) => screen = display.screen_map.get_mut(s).unwrap(),
    };

    let mut btn = None;
    for button in &mut screen.buttons {
        if button.id.clone().unwrap_or("DNE".to_string()) == col_or_type.0 {
            button.default_texture = col_or_type.1.clone();
        }
        if button.id.clone().unwrap_or("DNE".to_string())
            == button_id.clone().unwrap_or("dne ".to_string())
        {
            btn = Some(button);
        }
    }

    match btn {
        None => {}
        Some(b) => {
            let prev_id = b.id.clone().unwrap().clone();
            let prev_tex = b.default_texture.clone();
            if color_or_type {
                display.customization_choices.prev_color_selection = (prev_id, prev_tex);
            } else {
                display.customization_choices.prev_type_selection = (prev_id, prev_tex);
            }
            b.default_texture = b.hover_texture.clone();
            b.selected = true;
        }
    }
}
