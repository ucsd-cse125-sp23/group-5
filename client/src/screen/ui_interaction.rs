use crate::inputs::ClientSync::Ready;
use crate::inputs::{ClientSync, Input};
use crate::screen;

use super::objects::Icon;
use common::configs::display_config::ScreenLocation;

use crate::screen::Screen;
use common::core::choices::CurrentSelections;
use common::core::mesh_color::MeshColor;
use log::warn;
use nalgebra_glm as glm;
use phf::phf_map;
use common::core::choices::{OBJECT_PLAYER_MODEL, LEAF_MESH, BODY_MESH, DEFAULT_MODEL};

// pub const OBJECT_PLAYER_MODEL: &str = "object:player_model";
// pub const LEAF_MESH: &str = "leaf";
// pub const BODY_MESH: &str = "korok";
// pub const DEFAULT_MODEL: &str = "korok_1";

pub static BUTTON_MAP: phf::Map<&'static str, fn(&mut screen::Display, Option<String>)> = phf_map! {
    "game_start" => game_start,
    "change_leaf_type" => change_leaf_type,
    "change_leaf_color" => change_leaf_color,
    "change_wood_color" => change_wood_color,
    "go_to_lobby" => go_to_lobby,
    "go_to_title" => go_to_title,
};

// Place click events here ----------------------
fn game_start(display: &mut screen::Display, _: Option<String>) {
    // do nothing if no colors were selected
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display
        .screen_map
        .get_mut(&curr_group.screen.clone().unwrap())
        .unwrap();
    let sel_2 = selected_2colors(
        &display.customization_choices.final_choices.color,
        curr_screen,
        false,
    );

    if sel_2 && !display.customization_choices.ready {
        let ind = *curr_screen.btn_id_map.get("start_game").unwrap();
        curr_screen.buttons[ind].default_tint = nalgebra_glm::Vec4::new(0.0, 0.55, 0.0, 1.0);
        curr_screen.buttons[ind].hover_tint = nalgebra_glm::Vec4::new(0.0, 0.55, 0.0, 1.0);

        display.customization_choices.ready = true;

        let final_choices = display.customization_choices.final_choices.clone();
        println!("{:#?}", final_choices);

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
        // display.change_to(display.game_display.clone());
    }
}

fn go_to_title(display: &mut screen::Display, _: Option<String>) {
    display.change_to("display:title".to_owned());
}

fn go_to_lobby(display: &mut screen::Display, _: Option<String>) {
    display.change_to("display:lobby".to_owned());

    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display
        .screen_map
        .get_mut(&curr_group.screen.clone().unwrap())
        .unwrap();

    // reset selectors
    let curr_leaf_type: &mut Icon =
        &mut curr_screen.icons[*curr_screen.icon_id_map.get("leaf_type_selector").unwrap()];
    curr_leaf_type.location = ScreenLocation {
        vert_disp: (0.0, 0.555),
        horz_disp: (0.0, -1.333),
    };

    for i in vec!["leaf_color_selector", "wood_color_selector"] {
        let ind = *curr_screen.icon_id_map.get(i).unwrap();
        let icon = &mut curr_screen.icons[ind];
        icon.location = ScreenLocation {
            vert_disp: (1000.0, 1000.0),
            horz_disp: (1000.0, 1000.0),
        };
    }

    // reset go button
    let ind = *curr_screen.btn_id_map.get("start_game").unwrap();
    let def_col = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0);
    curr_screen.buttons[ind].default_tint = def_col;
    curr_screen.buttons[ind].hover_tint = def_col;

    // reset choices
    unselect_button(&display.customization_choices.curr_leaf_type, curr_screen);
    unselect_button(&display.customization_choices.curr_leaf_color, curr_screen);
    unselect_button(&display.customization_choices.curr_wood_color, curr_screen);

    let ind = *curr_screen.btn_id_map.get(DEFAULT_MODEL).unwrap();
    curr_screen.buttons[ind].selected = true;
    display.customization_choices = CurrentSelections::default();

    // reset model
    if let Some(scene) = display
        .scene_map
        .get_mut(&curr_group.scene.clone().unwrap())
    {
        if let Some(node) = scene.scene_graph.get_mut(OBJECT_PLAYER_MODEL) {
            let default_color = MeshColor::new([0.5, 0.5, 0.5]);
            let default_color_l = MeshColor::new([0.6, 0.6, 0.6]);
            display.customization_choices.final_choices.color.clear();
            display
                .customization_choices
                .final_choices
                .materials
                .clear();
            display
                .customization_choices
                .final_choices
                .color
                .insert(BODY_MESH.to_string(), default_color);
            display
                .customization_choices
                .final_choices
                .color
                .insert(LEAF_MESH.to_string(), default_color_l);
            node.model = Some(DEFAULT_MODEL.to_string());
            node.colors = Some(display.customization_choices.final_choices.color.clone());
            if let Some(mtls) = &mut node.materials {
                mtls.clear();
            }

            // reset position
            let rot = glm::Quat::new(0.5079111, -0.2949345, -0.7971848, -0.13986267);
            node.transform = glm::TMat4::<f32>::new_translation(&glm::Vec3::new(0.0, -0.25, 1.4))
                * glm::quat_to_mat4(&rot);
        }
        scene.draw_scene_dfs();
    }
}

fn change_leaf_type(display: &mut screen::Display, button_id: Option<String>) {
    if display.customization_choices.ready {
        return;
    }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display
        .screen_map
        .get_mut(&curr_group.screen.clone().unwrap())
        .unwrap();

    unselect_button(&display.customization_choices.curr_leaf_type, curr_screen);
    let curr_button = &mut curr_screen.buttons[*curr_screen
        .btn_id_map
        .get(&button_id.clone().unwrap())
        .unwrap()];
    display.customization_choices.curr_leaf_type = button_id.clone().unwrap();
    curr_button.selected = true;

    let curr_leaf_type =
        &mut curr_screen.icons[*curr_screen.icon_id_map.get("leaf_type_selector").unwrap()];

    if let Some(scene) = display
        .scene_map
        .get_mut(&curr_group.scene.clone().unwrap())
    {
        if let Some(node) = scene.scene_graph.get_mut(OBJECT_PLAYER_MODEL) {
            display.customization_choices.final_choices.model = button_id.clone().unwrap();
            node.model = Some(button_id.clone().unwrap());
            curr_leaf_type.location = curr_button.location.clone();
        }
        scene.draw_scene_dfs();
    }
}

fn change_leaf_color(display: &mut screen::Display, button_id: Option<String>) {
    if display.customization_choices.ready {
        return;
    }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display
        .screen_map
        .get_mut(&curr_group.screen.clone().unwrap())
        .unwrap();

    unselect_button(&display.customization_choices.curr_leaf_color, curr_screen);
    let curr_button = &mut curr_screen.buttons[*curr_screen
        .btn_id_map
        .get(&button_id.clone().unwrap())
        .unwrap()];
    display.customization_choices.curr_leaf_color = button_id.clone().unwrap();
    curr_button.selected = true;

    let curr_leaf_color =
        &mut curr_screen.icons[*curr_screen.icon_id_map.get("leaf_color_selector").unwrap()];

    let actual_color = MeshColor::new([
        curr_button.default_tint[0],
        curr_button.default_tint[1],
        curr_button.default_tint[2],
    ]);
    let actual_mtl = button_id.clone().unwrap();

    if let Some(scene) = display
        .scene_map
        .get_mut(&curr_group.scene.clone().unwrap())
    {
        if let Some(node) = scene.scene_graph.get_mut(OBJECT_PLAYER_MODEL) {
            display
                .customization_choices
                .final_choices
                .color
                .insert(LEAF_MESH.to_owned(), actual_color);
            display
                .customization_choices
                .final_choices
                .materials
                .insert(LEAF_MESH.to_owned(), actual_mtl);
            node.colors = Some(display.customization_choices.final_choices.color.clone());
            node.materials = Some(
                display
                    .customization_choices
                    .final_choices
                    .materials
                    .clone(),
            );
            curr_leaf_color.location = curr_button.location;
        }
        scene.draw_scene_dfs();
    }

    selected_2colors(
        &display.customization_choices.final_choices.color,
        curr_screen,
        true,
    );
}

fn change_wood_color(display: &mut screen::Display, button_id: Option<String>) {
    if display.customization_choices.ready {
        return;
    }
    let curr_group = display.groups.get_mut(&display.current).unwrap();
    let curr_screen = display
        .screen_map
        .get_mut(&curr_group.screen.clone().unwrap())
        .unwrap();

    unselect_button(&display.customization_choices.curr_wood_color, curr_screen);
    let curr_button = &mut curr_screen.buttons[*curr_screen
        .btn_id_map
        .get(&button_id.clone().unwrap())
        .unwrap()];
    display.customization_choices.curr_wood_color = button_id.clone().unwrap();
    curr_button.selected = true;

    let curr_wood_color =
        &mut curr_screen.icons[*curr_screen.icon_id_map.get("wood_color_selector").unwrap()];

    let actual_color = MeshColor::new([
        curr_button.default_tint[0],
        curr_button.default_tint[1],
        curr_button.default_tint[2],
    ]);
    let actual_mtl = button_id.clone().unwrap();

    if let Some(scene) = display
        .scene_map
        .get_mut(&curr_group.scene.clone().unwrap())
    {
        if let Some(node) = scene.scene_graph.get_mut(OBJECT_PLAYER_MODEL) {
            display
                .customization_choices
                .final_choices
                .color
                .insert(BODY_MESH.to_owned(), actual_color);
            display
                .customization_choices
                .final_choices
                .materials
                .insert(BODY_MESH.to_owned(), actual_mtl);
            node.colors = Some(display.customization_choices.final_choices.color.clone());
            node.materials = Some(
                display
                    .customization_choices
                    .final_choices
                    .materials
                    .clone(),
            );
            curr_wood_color.location = curr_button.location;
        }
        scene.draw_scene_dfs();
    }

    selected_2colors(
        &display.customization_choices.final_choices.color,
        curr_screen,
        true,
    );
}
// end click events ----------------------

fn selected_2colors(
    colors: &std::collections::HashMap<String, MeshColor>,
    curr_screen: &mut Screen,
    change_color: bool,
) -> bool {
    let mut len = 0;
    let default_color = [0.5, 0.5, 0.5];
    for (_, v) in colors {
        if v.rgb_color != default_color {
            len += 1;
        }
    }
    if len >= 2 {
        if change_color {
            let ind = *curr_screen.btn_id_map.get("start_game").unwrap();
            curr_screen.buttons[ind].default_tint = nalgebra_glm::Vec4::new(0.8, 0.0, 0.0, 1.0);
            curr_screen.buttons[ind].hover_tint = nalgebra_glm::Vec4::new(0.0, 0.55, 0.0, 1.0);
        }
        return true;
    }
    false
}

fn unselect_button(btn: &str, curr_screen: &mut Screen) {
    let ind = curr_screen.btn_id_map.get(btn);
    if let Some(i) = ind {
        curr_screen.buttons[*i].selected = false;
    }
}
