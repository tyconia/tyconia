//! Lightweight tooltip system.
//!
//! Attach a [`Tooltip`] component to any hoverable UI entity. After a short
//! delay a single floating panel is shown near the cursor. The panel is a
//! root-level UI node so it is positioned relative to the viewport and can
//! never be clipped by the widget that owns the tooltip.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TooltipState>().add_systems(
            Update,
            (
                ensure_tooltip_panel,
                track_hovered_tooltip,
                update_tooltip_panel.run_if(resource_exists::<TooltipPanelEntities>),
                position_tooltip_panel.run_if(resource_exists::<TooltipPanelEntities>),
            )
                .chain(),
        );
    }
}

/// Attach to a hoverable UI entity to show a tooltip after [`Tooltip::delay`].
#[derive(Component, Debug, Clone)]
#[require(Node, Interaction, RelativeCursorPosition)]
pub struct Tooltip {
    pub text: String,
    pub delay: f32,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            delay: TOOLTIP_DELAY,
        }
    }

    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            text: String::new(),
            delay: TOOLTIP_DELAY,
        }
    }
}

#[derive(Resource, Default)]
struct TooltipState {
    hovered: Option<Entity>,
    elapsed: f32,
    visible: bool,
}

#[derive(Component)]
struct TooltipPanel;

#[derive(Component)]
struct TooltipPanelText;

#[derive(Resource)]
struct TooltipPanelEntities {
    panel: Entity,
    text: Entity,
}

fn ensure_tooltip_panel(
    mut commands: Commands,
    fonts: Option<Res<FontAssets>>,
    ui: Option<Res<UiAssets>>,
    existing: Query<(), With<TooltipPanel>>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    let (Some(fonts), Some(ui)) = (fonts, ui) else {
        return;
    };

    let text_entity = commands
        .spawn((
            TooltipPanelText,
            Text::new(""),
            TextFont {
                font: fonts.jersey_25.clone(),
                font_size: SMALL_MEDIUM_FONT,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            },
            TextColor(Color::WHITE),
        ))
        .id();

    let panel = commands
        .spawn((
            TooltipPanel,
            GlobalZIndex::from(ZIndices::Tooltip),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: TOOLTIP_PADDING.rect_all(),
                max_width: Val::Px(TOOLTIP_MAX_WIDTH),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.96)),
            ImageNode {
                image: ui.window_content.clone(),
                image_mode: BUTTON_IMG_MODE_SLICED,
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    commands.entity(panel).add_child(text_entity);
    commands.insert_resource(TooltipPanelEntities {
        panel,
        text: text_entity,
    });
}

fn track_hovered_tooltip(
    query: Query<(Entity, &Interaction, &Tooltip)>,
    mut state: ResMut<TooltipState>,
) {
    let hovered = query
        .iter()
        .find(|(_, interaction, _)| **interaction == Interaction::Hovered)
        .map(|(entity, _, _)| entity);

    if hovered != state.hovered {
        state.hovered = hovered;
        state.elapsed = 0.0;
        state.visible = false;
    }
}

fn update_tooltip_panel(
    time: Res<Time>,
    mut state: ResMut<TooltipState>,
    query: Query<&Tooltip>,
    mut panels: Query<&mut Visibility, With<TooltipPanel>>,
    mut texts: Query<&mut Text, With<TooltipPanelText>>,
    entities: Res<TooltipPanelEntities>,
) {
    let Ok(mut visibility) = panels.get_mut(entities.panel) else {
        return;
    };

    let Some(hovered) = state.hovered else {
        *visibility = Visibility::Hidden;
        state.visible = false;
        return;
    };

    let Ok(tooltip) = query.get(hovered) else {
        state.hovered = None;
        *visibility = Visibility::Hidden;
        state.visible = false;
        return;
    };

    if !state.visible {
        state.elapsed += time.delta_secs();
        if state.elapsed >= tooltip.delay {
            if let Ok(mut text) = texts.get_mut(entities.text) {
                *text = Text::new(tooltip.text.clone());
            }
            *visibility = Visibility::Visible;
            state.visible = true;
        }
    }
}

fn position_tooltip_panel(
    state: Res<TooltipState>,
    windows: Query<&Window>,
    mut panels: Query<(&mut Node, &ComputedNode), With<TooltipPanel>>,
    entities: Res<TooltipPanelEntities>,
) {
    if !state.visible {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((mut node, computed)) = panels.get_mut(entities.panel) else {
        return;
    };

    const OFFSET: f32 = 16.0;
    let size = computed.size() * computed.inverse_scale_factor();
    let width = if size.x > 0.0 {
        size.x
    } else {
        TOOLTIP_MAX_WIDTH
    };
    let height = size.y;

    let mut left = cursor.x + OFFSET;
    let mut top = cursor.y + OFFSET;

    if left + width > window.width() {
        left = (cursor.x - width - OFFSET).max(0.0);
    }
    if height > 0.0 && top + height > window.height() {
        top = (window.height() - height - OFFSET).max(0.0);
    }

    node.left = Val::Px(left);
    node.top = Val::Px(top);
}