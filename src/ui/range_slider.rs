use crate::loading::*;
use crate::ui::*;
use bevy::ui::widget::NodeImageMode;
use std::ops::Range;

const SLIDER_TRACK_HEIGHT: f32 = 8.0;
const SLIDER_THUMB_HEIGHT: f32 = 40.0;
const SLIDER_THUMB_TRAVEL: f32 = 90.0;

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

pub fn labeled_slider<C: Bundle>(
    cmd: &mut ChildBuilder,
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

pub fn draw_slider<C: Bundle>(
    cmd: &mut ChildBuilder,
    components: C,
    ui: &Res<UiAssets>,
    slider: Slider,
) {
    cmd.spawn((
        Node {
            height: Val::Px(SLIDER_TRACK_HEIGHT),
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
            hover: ui.range_slider_thumb_hover.clone(),
        };

        parent
            .spawn((
                super::DepressButton::default(),
                Node {
                    height: Val::Px(SLIDER_THUMB_HEIGHT),
                    aspect_ratio: Some(4. / 3.),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(slider.percentage / 100. * SLIDER_THUMB_TRAVEL),
                    top: Val::Px(-(SLIDER_THUMB_HEIGHT - SLIDER_TRACK_HEIGHT) / 2.0),
                    ..default()
                },
                slider,
                ImageNode {
                    image: button_skins.normal.clone(),
                    image_mode: BUTTON_IMG_MODE_SLICED,
                    ..default()
                },
                button_skins,
                components,
            ))
            .observe(drag_slider);
    });
}

fn drag_slider(
    trigger: Trigger<Pointer<Drag>>,
    mut sliders: Query<(&mut Slider, &mut Node, &Parent)>,
    tracks: Query<&ComputedNode>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok((mut slider, mut node, parent)) = sliders.get_mut(trigger.entity()) else {
        return;
    };
    let Ok(track) = tracks.get(parent.get()) else {
        return;
    };

    let travel = track.size().x * track.inverse_scale_factor() * (SLIDER_THUMB_TRAVEL / 100.0);
    if travel <= 0.0 {
        return;
    }

    let delta = trigger.delta.x / travel * 100.0;
    slider.percentage = (slider.percentage + delta).clamp(0.0, 100.0);
    node.left = Val::Percent(slider.percentage / 100.0 * SLIDER_THUMB_TRAVEL);
}
