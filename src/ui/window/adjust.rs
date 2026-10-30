use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::SystemCursorIcon;

use crate::ui::*;
use bevy::input::mouse::*;

pub struct WindowUIAdjustmentPlugin;

impl Plugin for WindowUIAdjustmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (scroll_window,).run_if(any_with_component::<Scrollable>),
        );
    }
}

#[derive(Debug, Component)]
pub struct WindowResize(pub WindowResizeSource);

/// Window component that gets resized from
#[derive(Debug)]
pub enum WindowResizeSource {
    Top,
    TopLeft,
    TopRight,
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Component)]
pub struct WindowTitleBar;

pub fn scroll_window(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &RelativeCursorPosition), With<self::Scrollable>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll_position, cursor) in query.iter_mut() {
            if cursor.mouse_over() {
                let scroll_amount = match event.unit {
                    MouseScrollUnit::Line => event.y * 20.0,
                    MouseScrollUnit::Pixel => event.y,
                };

                scroll_position.offset_y += scroll_amount;
            }
        }
    }
}

fn resize_source(
    relative_cursor_pos: &RelativeCursorPosition,
) -> Option<(WindowResizeSource, SystemCursorIcon)> {
    let pos = relative_cursor_pos.normalized?;
    let threshold = 0.01;
    let near_edges = (
        pos.x < threshold,
        pos.x > 1.0 - threshold,
        pos.y < threshold,
        pos.y > 1.0 - threshold,
    );

    match near_edges {
        (true, false, true, false) => {
            Some((WindowResizeSource::TopLeft, SystemCursorIcon::NwResize))
        }
        (false, true, true, false) => {
            Some((WindowResizeSource::TopRight, SystemCursorIcon::NeResize))
        }
        (true, false, false, true) => {
            Some((WindowResizeSource::BottomLeft, SystemCursorIcon::SwResize))
        }
        (false, true, false, true) => {
            Some((WindowResizeSource::BottomRight, SystemCursorIcon::SeResize))
        }
        (true, false, false, false) => Some((WindowResizeSource::Left, SystemCursorIcon::WResize)),
        (false, true, false, false) => Some((WindowResizeSource::Right, SystemCursorIcon::EResize)),
        (false, false, true, false) => Some((WindowResizeSource::Top, SystemCursorIcon::NResize)),
        (false, false, false, true) => Some((WindowResizeSource::Bottom, SystemCursorIcon::SResize)),
        _ => None,
    }
}

pub fn window_resize_start(
    trigger: Trigger<Pointer<Down>>,
    mut commands: Commands,
    mut windows: Query<(Entity, &RelativeCursorPosition), With<WindowUI>>,
    #[cfg(not(target_arch = "wasm32"))] mut cursor: CurrentCursorChannel,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok((entity, relative_cursor_pos)) = windows.get_mut(trigger.entity()) else {
        return;
    };

    commands.entity(entity).remove::<WindowResize>();

    let Some((source, icon)) = resize_source(relative_cursor_pos) else {
        #[cfg(not(target_arch = "wasm32"))]
        cursor.send(SystemCursorIcon::Default.into());
        return;
    };

    commands.entity(entity).insert(WindowResize(source));

    #[cfg(not(target_arch = "wasm32"))]
    cursor.send(icon.into());
}

pub fn window_resize_end(
    trigger: Trigger<Pointer<Up>>,
    mut commands: Commands,
    windows: Query<Entity, (With<WindowUI>, With<WindowResize>)>,
    #[cfg(not(target_arch = "wasm32"))] mut cursor: CurrentCursorChannel,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok(entity) = windows.get(trigger.entity()) else {
        return;
    };

    commands.entity(entity).remove::<WindowResize>();

    #[cfg(not(target_arch = "wasm32"))]
    cursor.send(SystemCursorIcon::Default.into());
}

/// WindowDragged is attached to the child entity that has WindowTitleBar,
/// we need to access the parent which is the window itself
pub fn window_title_drag(
    trigger: Trigger<Pointer<Drag>>,
    titles: Query<&Parent, With<WindowTitleBar>>,
    mut ui_windows: Query<(&mut Node, Option<&WindowResize>), With<WindowUI>>,
    viewports: Query<&Window>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok(parent) = titles.get(trigger.entity()) else {
        return;
    };
    let Ok((mut node, resize)) = ui_windows.get_mut(parent.get()) else {
        return;
    };

    if resize.is_some() {
        return;
    }

    let Ok(window) = viewports.get_single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());

    let window_width = node.width.resolve(0., viewport).unwrap_or(0.);
    let window_height = node.height.resolve(0., viewport).unwrap_or(0.);

    if let Val::Px(ref mut px) = node.left {
        *px = (*px + trigger.delta.x).clamp(0., (viewport.x - window_width).max(0.));
    }
    if let Val::Px(ref mut px) = node.top {
        *px = (*px + trigger.delta.y).clamp(0., (viewport.y - window_height).max(0.));
    }
}

pub fn window_resize_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut ui_windows: Query<(&mut Node, &WindowResize), With<WindowUI>>,
    viewports: Query<&Window>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let Ok((mut node, resize)) = ui_windows.get_mut(trigger.entity()) else {
        return;
    };
    let Ok(window) = viewports.get_single() else {
        return;
    };

    let viewport_width = window.width();
    let viewport_height = window.height();
    let viewport = Vec2::new(viewport_width, viewport_height);

    let current_left = node.left.resolve(0., viewport).unwrap_or(0.);
    let current_top = node.top.resolve(0., viewport).unwrap_or(0.);
    let current_width = node.width.resolve(0., viewport).unwrap_or(0.);
    let current_height = node.height.resolve(0., viewport).unwrap_or(0.);

    let mut new_left = current_left;
    let mut new_top = current_top;
    let mut new_width = current_width;
    let mut new_height = current_height;

    let inlined_movement = trigger.delta.x;
    let block_movement = trigger.delta.y;

    match resize.0 {
        WindowResizeSource::TopLeft => {
            new_left += inlined_movement;
            new_width -= inlined_movement;
            new_top += block_movement;
            new_height -= block_movement;
        }
        WindowResizeSource::TopRight => {
            new_top += block_movement;
            new_height -= block_movement;
            new_width += inlined_movement;
        }
        WindowResizeSource::BottomLeft => {
            new_height += block_movement;
            new_left += inlined_movement;
            new_width -= inlined_movement;
        }
        WindowResizeSource::BottomRight => {
            new_height += block_movement;
            new_width += inlined_movement;
        }
        WindowResizeSource::Left => {
            new_left += inlined_movement;
            new_width -= inlined_movement;
        }
        WindowResizeSource::Top => {
            new_top += block_movement;
            new_height -= block_movement;
        }
        WindowResizeSource::Right => {
            new_width += inlined_movement;
        }
        WindowResizeSource::Bottom => {
            new_height += block_movement;
        }
    }

    // Preserve opposite edge when resizing from left/top
    let right = current_left + current_width;
    let bottom = current_top + current_height;

    let resizing_left = matches!(
        resize.0,
        WindowResizeSource::Left | WindowResizeSource::TopLeft | WindowResizeSource::BottomLeft
    );
    let resizing_top = matches!(
        resize.0,
        WindowResizeSource::Top | WindowResizeSource::TopLeft | WindowResizeSource::TopRight
    );

    new_width = new_width.max(WINDOW_MIN_WIDTH);
    new_height = new_height.max(WINDOW_MIN_HEIGHT);

    if resizing_left {
        new_left = right - new_width;
    }
    if resizing_top {
        new_top = bottom - new_height;
    }

    // Viewport constraints
    if new_left < 0. {
        new_left = 0.;
        if resizing_left {
            new_width = right.min(viewport_width);
        }
    }
    if new_top < 0. {
        new_top = 0.;
        if resizing_top {
            new_height = bottom.min(viewport_height);
        }
    }

    if new_left + new_width > viewport_width {
        if resizing_left {
            new_left = (viewport_width - new_width).max(0.);
        } else {
            new_width = (viewport_width - new_left).max(WINDOW_MIN_WIDTH);
        }
    }
    if new_top + new_height > viewport_height {
        if resizing_top {
            new_top = (viewport_height - new_height).max(0.);
        } else {
            new_height = (viewport_height - new_top).max(WINDOW_MIN_HEIGHT);
        }
    }

    // Final clamp against viewport
    new_width = new_width.min(viewport_width);
    new_height = new_height.min(viewport_height);

    if let Val::Px(ref mut px) = node.left {
        *px = new_left;
    }
    if let Val::Px(ref mut px) = node.top {
        *px = new_top;
    }
    if let Val::Px(ref mut px) = node.width {
        *px = new_width;
    }
    if let Val::Px(ref mut px) = node.height {
        *px = new_height;
    }
}
