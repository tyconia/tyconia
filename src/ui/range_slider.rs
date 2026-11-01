use crate::loading::*;
use crate::ui::*;
use bevy::ui::widget::NodeImageMode;
use bevy::ui::RelativeCursorPosition;
use std::ops::Range;

const SLIDER_TRACK_HEIGHT: f32 = 8.0;
const SLIDER_THUMB_HEIGHT: f32 = 40.0;

pub struct RangeSliderPlugin;

impl Plugin for RangeSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, layout_slider_thumbs);
    }
}

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
        let percentage = Self::percent_range(&range, value).clamp(0.0, 100.0);
        let mut slider = Self {
            percentage,
            range,
            steps,
        };
        slider.snap_to_step();
        slider
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
        if range.end <= range.start {
            return 0.0;
        }
        ((value - range.start) / (range.end - range.start)) * 100.0
    }

    pub fn percentage_from_fraction(&self, fraction: f32) -> f32 {
        fraction.clamp(0.0, 1.0) * 100.0
    }

    pub fn set_percentage(&mut self, percentage: f32) -> bool {
        let percentage = percentage.clamp(0.0, 100.0);
        if (percentage - self.percentage).abs() < f32::EPSILON {
            return false;
        }
        self.percentage = percentage;
        self.snap_to_step();
        true
    }

    fn snap_to_step(&mut self) {
        if self.steps <= 1 {
            return;
        }
        let increment = 100.0 / (self.steps - 1) as f32;
        self.percentage = (self.percentage / increment).round() * increment;
    }
}

#[derive(Component, Debug)]
pub struct SliderTrack;

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
        align_items: AlignItems::Center,
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
    let mut track = cmd.spawn((
        SliderTrack,
        Node {
            height: Val::Px(SLIDER_TRACK_HEIGHT),
            width: Val::Percent(40.),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            ..default()
        },
        Interaction::default(),
        RelativeCursorPosition::default(),
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
    ));

    track.observe(slider_track_click).with_children(|parent| {
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
                    left: Val::Percent(slider.percentage),
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

fn slider_track_click(
    trigger: Trigger<Pointer<Click>>,
    tracks: Query<(&RelativeCursorPosition, &Children, &ComputedNode)>,
    mut sliders: Query<(&mut Slider, &mut Node, &ComputedNode)>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok((cursor, children, track)) = tracks.get(trigger.entity()) else {
        return;
    };
    let Some(normalized) = cursor.normalized else {
        return;
    };
    let track_width = track.size().x;

    for child in children.iter() {
        let Ok((mut slider, mut node, thumb)) = sliders.get_mut(*child) else {
            continue;
        };
        let available = track_width - thumb.size().x;
        if available <= 0.0 {
            continue;
        }
        let pointer = normalized.x * track_width;
        let fraction = (pointer - thumb.size().x * 0.5) / available;
        let percentage = slider.percentage_from_fraction(fraction);
        if slider.set_percentage(percentage) {
            node.left = Val::Percent(slider.percentage);
        }
    }
}

fn drag_slider(
    trigger: Trigger<Pointer<Drag>>,
    mut sliders: Query<(&mut Slider, &mut Node, &Parent, &ComputedNode)>,
    tracks: Query<&ComputedNode, With<SliderTrack>>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok((mut slider, mut node, parent, thumb)) = sliders.get_mut(trigger.entity()) else {
        return;
    };
    let Ok(track) = tracks.get(parent.get()) else {
        return;
    };

    let scale = thumb.inverse_scale_factor();
    let available = (track.size().x - thumb.size().x) * scale;
    if available <= 0.0 {
        return;
    }

    let delta = trigger.delta.x / available * 100.0;
    let percentage = slider.percentage + delta;
    if slider.set_percentage(percentage) {
        node.left = Val::Percent(slider.percentage);
    }
}

fn layout_slider_thumbs(
    mut sliders: Query<
        (&Slider, &mut Node, &Parent, &ComputedNode),
        Or<(Changed<Slider>, Changed<ComputedNode>)>,
    >,
    tracks: Query<&ComputedNode, With<SliderTrack>>,
) {
    for (slider, mut node, parent, thumb) in sliders.iter_mut() {
        let Ok(track) = tracks.get(parent.get()) else {
            continue;
        };
        let scale = thumb.inverse_scale_factor();
        let available = (track.size().x - thumb.size().x) * scale;
        let left = available.max(0.0) * slider.percentage / 100.0;
        if node.left != Val::Px(left) {
            node.left = Val::Px(left);
        }
        let top = -(SLIDER_THUMB_HEIGHT - SLIDER_TRACK_HEIGHT) / 2.0;
        if node.top != Val::Px(top) {
            node.top = Val::Px(top);
        }
    }
}
