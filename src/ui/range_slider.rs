use crate::loading::*;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use std::ops::Range;

pub struct RangeSliderPlugin;

impl Plugin for RangeSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_range_slider
                    .run_if(on_event::<CursorMoved>.and(any_with_component::<Slider>)),
                interact_range_slider.run_if(any_with_component::<Slider>),
            ),
        );
    }
}

#[derive(Component)]
/// Marker for active sliders
pub struct SliderDragged;

#[derive(Component, Debug)]
/// Textured horizontal range_slider with scroll support
pub struct Slider {
    pub steps: usize,
    pub range: Range<f32>,
    /// 0-100. percentage
    pub percentage: f32,
}

impl Slider {
    pub fn new(range: Range<f32>, value: f32, steps: usize) -> Self {
        Self {
            percentage: Self::percent_range(&range, value),
            range,
            steps,
        }
    }

    /// Converts slider value to range proportions
    pub fn valued(&self) -> f32 {
        Self::value_range(&self.range, self.percentage)
    }

    // converts percentage to range proportions
    pub fn value_range(range: &Range<f32>, value: f32) -> f32 {
        range.start + (value / 100.) * (range.end - range.start)
    }

    pub fn percent_range(range: &Range<f32>, value: f32) -> f32 {
        ((value - range.start) / (range.end - range.start)) * 100.0
    }
}

pub fn labeled_slider<T: ChildBuild, C: Bundle>(
    cmd: &mut T,
    name: &str,
    components: C,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,

    slider: Slider,
) {
    cmd.spawn((Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    },))
        .with_children(|parent| {
            super::body_text(name, parent, fonts);
            draw_slider(parent, components, ui, slider);
        });
}

pub fn draw_slider<T: ChildBuild, C: Bundle>(
    cmd: &mut T,
    components: C,
    ui: &Res<UiAssets>,
    slider: Slider,
) {
    cmd.spawn((
        Node {
            height: Val::Px(8.),
            width: Val::Percent(40.),
            position_type: PositionType::Relative,
            ..default()
        },
        ImageNode {
            image: ui.range_slider_track.clone(),
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::square(1.0),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 4.0 },
                max_corner_scale: 4.0,
                ..default()
            }),
            ..default()
        },
    ))
    .with_children(|parent| {
        let button_skins = super::ButtonSkins {
            active: ui.range_slider_thumb_active.clone(),
            normal: ui.range_slider_thumb.clone(),
        };

        parent.spawn((
            super::DepressButton::default(),
            Node {
                height: Val::Px(20.),
                aspect_ratio: Some(16. / 9.),
                position_type: PositionType::Absolute,
                left: Val::Percent(slider.percentage / 100. * 90.),
                top: Val::Percent(-100.0), // Position top edge at 50% of parent height
                ..default()
            },
            slider,
            ImageNode {
                image: button_skins.normal.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::from([3., 3., 1., 0.]),
                    center_scale_mode: SliceScaleMode::Tile { stretch_value: 4. },
                    sides_scale_mode: SliceScaleMode::Tile { stretch_value: 4. },

                    max_corner_scale: 4.0,
                }),
                ..default()
            },
            button_skins,
            components,
        ));
    });
}

pub fn interact_range_slider(
    mut cmd: Commands,
    range_slider: Query<
        (Entity, &super::DepressButton, Option<&SliderDragged>),
        (Changed<super::DepressButton>, With<Slider>),
    >,
) {
    for (slider, depress, dragged) in range_slider.iter() {
        if depress.held() {
            info!("slider activated");
            cmd.entity(slider).insert(SliderDragged);
        } else {
            dragged.map(|_| {
                info!("slider de-activated");
                cmd.entity(slider).remove::<SliderDragged>();
            });
        }
    }
}

pub fn update_range_slider(
    window: Query<&Window, Without<Slider>>,
    camera: Query<(&Camera, &GlobalTransform), Without<Slider>>,
    mut slider: Query<(&mut Slider, &mut Node), With<SliderDragged>>,
    mut cursor: EventReader<CursorMoved>,
) {
    let (camera, camera_transform) = camera.single();

    if let Some(Ok(cursor_world)) = window
        .single()
        .cursor_position()
        .and_then(|cur_pos| Some(camera.viewport_to_world_2d(camera_transform, cur_pos)))
    {
        if slider.iter().count() == 1 {
            let (mut slider, mut node) = slider.single_mut();
            info!("Cursor is positioned at {}", cursor_world,);

            let delta = cursor
                .read()
                .filter_map(|cm| cm.delta)
                .fold(0., |acc, delta| acc + delta.x);

            const FACTOR: f32 = 0.4;
            slider.percentage = slider.percentage as f32 + delta * FACTOR;
            slider.percentage = slider.percentage.clamp(0., 100.);
            node.left = Val::Percent(slider.percentage / 100. * 90.);
        }
    }

    // always consume event reader
    cursor.read();
}
