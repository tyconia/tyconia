use bevy::prelude::*;

use crate::ui::*;
use bevy::input::mouse::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::SystemCursorIcon;

pub struct WindowUIAdjustmentPlugin;

impl Plugin for WindowUIAdjustmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (scroll_window,).run_if(any_with_component::<Scrollable>),
                (flag_resize_window,).run_if(any_with_component::<WindowUI>),
                (flag_drag_window,).run_if(any_with_component::<WindowTitleBar>),
                (
                    (drag_window,).run_if(any_with_component::<WindowDragged>),
                    (resize_window,).run_if(any_with_component::<WindowResize>),
                ),
            ),
        );
    }
}
#[derive(Debug, Component)]
#[require(DepressButton, RelativeCursorPosition)]
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
#[require(ScrollPosition)]
pub struct Scrollable;

#[derive(Debug, Component)]
#[require(DepressButton)]
pub struct WindowTitleBar;

#[derive(Debug, Component)]
pub struct WindowDragged;

pub fn scroll_window(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(Entity, &mut ScrollPosition, &Interaction), With<self::Scrollable>>,
) {
    for event in mouse_wheel_events.read() {
        for (entity, mut scroll_position, interaction) in query.iter_mut() {
            if *interaction == Interaction::Hovered {
                let scroll_amount = match event.unit {
                    MouseScrollUnit::Line => event.y * 20.0,
                    MouseScrollUnit::Pixel => event.y,
                };

                scroll_position.offset_y += scroll_amount;
                info!(
                    "scroll position {} from {}",
                    scroll_position.offset_y, entity
                );
            }
        }
    }
}

/// WindowDragged is attached to the child entity that has WindowTitleBar,
/// we need to access the parent which is the window itself
pub fn drag_window(
    mut cmd: Commands,
    mut cursor_position: EventReader<CursorMoved>,
    window: Query<&Parent, With<WindowDragged>>,
) {
    // inlined is left, right;
    // block is top, bottom
    let (inlined_movement, block_movement) = cursor_position
        .read()
        .filter_map(|c| c.delta)
        .fold((0., 0.), |acc, xy| (acc.0 + xy.x, acc.1 + xy.y));

    for parent in window.iter() {
        cmd.entity(parent.get())
            .entry::<Node>()
            .and_modify(move |mut node| {
                if let Val::Px(ref mut px) = node.left {
                    *px += inlined_movement;
                }
                if let Val::Px(ref mut px) = node.top {
                    *px += block_movement;
                }
            });
    }

    cursor_position.read().count();
}

/// TODO: Dragging functionality
pub fn flag_drag_window(
    mut cmd: Commands,
    window: Query<(Entity, &DepressButton, Option<&WindowDragged>), With<WindowTitleBar>>,
) {
    for (entity, depress, already_dragging) in window.iter() {
        if depress.held() && !already_dragging.is_some() {
            info!("Window is dragged");
            cmd.entity(entity).insert(WindowDragged);
        } else if !depress.held() && already_dragging.is_some() {
            info!("Window is no longer dragged");
            cmd.entity(entity).remove::<WindowDragged>();
        }
    }
}

pub fn resize_window(
    mut cursor_position: EventReader<CursorMoved>,
    mut window: Query<(&mut Node, &WindowResize)>,
) {
    // inlined is left, right;
    // block is top, bottom
    let (inlined_movement, block_movement) = cursor_position
        .read()
        .filter_map(|c| c.delta)
        .fold((0., 0.), |acc, xy| (acc.0 + xy.x, acc.1 + xy.y));

    for (mut node, resize) in window.iter_mut() {
        match resize.0 {
            WindowResizeSource::TopLeft => {
                if let Val::Px(ref mut px) = node.left {
                    *px += inlined_movement;
                }
                if let Val::Px(ref mut px) = node.width {
                    *px -= inlined_movement;
                }
                if let Val::Px(ref mut px) = node.top {
                    *px += block_movement;
                }

                if let Val::Px(ref mut px) = node.height {
                    *px -= block_movement;
                }
            }
            WindowResizeSource::TopRight => {
                if let Val::Px(ref mut px) = node.top {
                    *px += block_movement;
                }

                if let Val::Px(ref mut px) = node.height {
                    *px -= block_movement;
                }

                if let Val::Px(ref mut px) = node.height {
                    *px += block_movement;
                }
            }
            WindowResizeSource::BottomLeft => {
                if let Val::Px(ref mut px) = node.height {
                    *px += block_movement;
                }
                if let Val::Px(ref mut px) = node.left {
                    *px += inlined_movement;
                }
                if let Val::Px(ref mut px) = node.width {
                    *px -= inlined_movement;
                }
            }
            WindowResizeSource::BottomRight => {
                if let Val::Px(ref mut px) = node.height {
                    *px += block_movement;
                }
                if let Val::Px(ref mut px) = node.width {
                    *px += inlined_movement;
                }
            }
            WindowResizeSource::Left => {
                if let Val::Px(ref mut px) = node.left {
                    *px += inlined_movement;
                }
                if let Val::Px(ref mut px) = node.width {
                    *px -= inlined_movement;
                }
            }
            WindowResizeSource::Top => {
                if let Val::Px(ref mut px) = node.top {
                    *px += block_movement;
                }

                if let Val::Px(ref mut px) = node.height {
                    *px -= block_movement;
                }
            }
            WindowResizeSource::Right => {
                if let Val::Px(ref mut px) = node.width {
                    *px += inlined_movement;
                }
            }
            WindowResizeSource::Bottom => {
                if let Val::Px(ref mut px) = node.height {
                    *px += block_movement;
                }
            }
        }
    }

    cursor_position.read().count();
}

pub fn flag_resize_window(
    mut cmd: Commands,
    ui_windows: Query<
        (
            Entity,
            &DepressButton,
            &RelativeCursorPosition,
            Option<&WindowResize>,
        ),
        With<WindowUI>,
    >,

    mut current_cursor_channel: CurrentCursorChannel,
    mut current_cursor: Local<Option<SystemCursorIcon>>,
) {
    current_cursor.map(|cursor| {
        current_cursor_channel.send(cursor.into());
    });

    for (entity, depress, relative_cursor_pos, already_resizing) in ui_windows.iter() {
        //info!("relative position {:?}", relative_cursor_pos);
        if let Some(pos) = relative_cursor_pos.normalized {
            let threshold = 0.01;
            let near_edges = (
                pos.x < threshold,
                pos.x > 1.0 - threshold,
                pos.y < threshold,
                pos.y > 1.0 - threshold,
            );

            let near_edge_position = match near_edges {
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
                (true, false, false, false) => {
                    Some((WindowResizeSource::Left, SystemCursorIcon::WResize))
                }
                (false, true, false, false) => {
                    Some((WindowResizeSource::Right, SystemCursorIcon::EResize))
                }
                (false, false, true, false) => {
                    Some((WindowResizeSource::Top, SystemCursorIcon::NResize))
                }
                (false, false, false, true) => {
                    Some((WindowResizeSource::Bottom, SystemCursorIcon::SResize))
                }
                _ => None,
            };

            match (
                near_edge_position,
                depress.held(),
                already_resizing.is_some(),
            ) {
               // case 1: flag resize on a held ui window & set cursor
                (Some((resize_source, resize_icon_cursor)), true, false) => {
                    //info!("Resizing {:?}", resize_source);

                    cmd.entity(entity).insert(WindowResize(resize_source));
                    *current_cursor = Some(resize_icon_cursor);
                }
                // case 2: unflag resize on a unheld ui window & reset cursor
                (_, false, true) => {
                    //info!("Window resize released");

                    cmd.entity(entity).remove::<WindowResize>();
                    *current_cursor = None;
                }
                // case 3: set cursor on hover
                //(Some((_, resize_icon_cursor)), false, false) => {
                //    *current_cursor = Some(resize_icon_cursor);
                //}
                // case 4: reset cursor off hover
                //(None, false, false) => {
                //    *current_cursor = None;
                //}
 

                _ => {}
            }
        }
    }
}
