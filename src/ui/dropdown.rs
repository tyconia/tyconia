//! Textured dropdown menu for multiple choice

use super::*;
use crate::{loading, *};
use bevy::prelude::*;

#[derive(Component)]
#[require(Node)]
pub struct Dropdown {
    // index of options
    pub selected: Option<usize>,
    pub options: Vec<String>,
    pub expanded: bool,
}

#[derive(Component)]
pub struct DropdownOption;

pub fn spawn_dropdown(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
    options: &[String],
    place_holder: String,
    selected: Option<usize>,
) {
    spawn_button(
        ButtonType::LabeledIcon {
            text: place_holder,
            icon: ui.dropdown_ico.clone(),
            font_size: UI_SCALE * 3.,
            image_size: Val::Px(UI_SCALE * 4.),
        } 
        , DropdownOption, parent, &fonts, &ui);
}


pub fn redraw_dropdown(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
    options: &[String],
    place_holder: String,
    selected: Option<usize>,
) {
    parent
        .spawn((
            Dropdown {
                selected,
                options: options.to_vec(),
                expanded: false,
            },
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(UI_SCALE),
                padding: UiRect::axes(Val::Px(UI_SCALE * 2.), Val::Px(UI_SCALE * 2.)),
                ..default()
            },
            DepressButton::default(),
            ButtonSkins {
                normal: ui.button_alpha.clone(),
                active: ui.button_alpha_active.clone(),
                hover: ui.button_alpha_hover.clone(),
            },
            ImageNode {
                image: ui.button_alpha.clone(),
                image_mode: crate::ui::BUTTON_IMG_MODE_SLICED,
                ..default()
            },
        ))
        .with_children(|parent| {
            let options_ = vec![place_holder];
            //options_.extend_from_slice(options);
            options_.into_iter().for_each(|option| {
                //spawn_button(option.into(), DropdownOption, &mut parent, &fonts, &ui);
                section_text(&option, parent, &fonts);
            });
        });
}
