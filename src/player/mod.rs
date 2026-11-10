use crate::actions::{movement::MovementAction, ui::UiAction};
use crate::levels::*;
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpawnRequest>()
            .add_systems(OnEnter(GameState::Playing), (spawn_player,))
            .add_systems(
                Update,
                (
                    apply_player_spawn,
                    move_player.run_if(on_event::<MovementAction>),
                    sync_player_visual,
                    zoom_camera,
                    //(un_highlight, (hightlight_building /* highlight_tile */,))
                    //    .chain()
                    //    .run_if(resource_changed::<CursorWorldPosition>),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerVisual;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerHeight {
    pub elevation: f32,
    pub rank: i16,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerCollider {
    pub radius: f32,
}

fn spawn_player(mut commands: Commands) {
    let mut inventory = crate::levels::Inventory::with_capacity(22);

    inventory.dump(vec![
        ItemEntry {
            item: COLLIDER_ITEM_ID.into(),
            quantity: INFINITE_QUANTITY,
        },
        ItemEntry {
            item: "auto_arm".into(),
            quantity: 20,
        },
        ItemEntry {
            item: "auto_arm".into(),
            quantity: 20,
        },
        ItemEntry {
            item: "mover_belt".into(),
            quantity: 80,
        },
        ItemEntry {
            item: "infinite_io".into(),
            quantity: 20,
        },
    ]);

    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 10.0),
            StateScoped(GameState::Playing),
            Player,
            PlayerHeight::default(),
            PlayerCollider { radius: 10.0 },
            inventory,
            crate::hud::InventoryUISource {
                display_width: 6,
                display_height: 5,
            },
            crate::levels::InventoryActive::default(),
        ))
        .with_children(|children| {
            children.spawn((
                PlayerVisual,
                Sprite::from_color(Color::srgb(0.95, 0.72, 0.2), Vec2::splat(22.0)),
                Transform::default(),
            ));
            children.spawn((
                Camera2d,
                Projection::Orthographic(OrthographicProjection {
                    near: -LEVEL_Z_BAND * 16.0,
                    far: LEVEL_Z_BAND * 16.0,
                    ..OrthographicProjection::default_2d()
                }),
                Msaa::Off,
                UiAntiAlias::Off,
                Transform::from_xyz(0.0, 0.0, 500.0),
            ));
        });
}

fn apply_player_spawn(
    mut spawn_request: ResMut<PlayerSpawnRequest>,
    grid: Option<Res<IsoGrid>>,
    current: Option<Res<CurrentLevel>>,
    verticals: Query<(&LevelInstance, &LevelVertical)>,
    mut player_query: Query<(&mut Transform, &mut PlayerHeight), With<Player>>,
) {
    let Some(position) = spawn_request.0.take() else {
        return;
    };
    let Ok((mut transform, mut height)) = player_query.get_single_mut() else {
        return;
    };

    transform.translation.x = position.x;
    transform.translation.y = position.y;

    let Some(grid) = grid else {
        return;
    };
    let Some(current) = current else {
        return;
    };
    let Some(vertical) = current
        .0
        .as_ref()
        .and_then(|label| {
            verticals
                .iter()
                .find(|(instance, _)| &instance.prefab == label)
        })
        .map(|(_, vertical)| vertical)
    else {
        return;
    };
    if let Some(sample) = vertical.sample(&grid, position, 0.0) {
        height.elevation = sample.elevation;
        height.rank = sample.rank;
    }
}

/// Zooms camera from Actions with eased interpolation
fn zoom_camera(
    mut ui_action: EventReader<UiAction>,
    mut camera: Query<&mut OrthographicProjection, With<Camera>>,
    mut target_scale: Local<Option<f32>>,
    time: Res<Time>,
) {
    const ZOOM_STEP: f32 = 0.15;
    const ZOOM_RESPONSE: f32 = 12.0;
    const MIN_SCALE: f32 = 0.25;
    const MAX_SCALE: f32 = 1.8;

    let factor: isize = ui_action
        .read()
        .filter_map(|ui_action| ui_action.zoom())
        .sum();

    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let mut target = target_scale.unwrap_or(camera.scale);
    target = (target - factor as f32 * ZOOM_STEP).clamp(MIN_SCALE, MAX_SCALE);

    let remaining = target - camera.scale;
    if remaining.abs() <= 0.0005 {
        camera.scale = target;
    } else {
        let blend = 1.0 - (-ZOOM_RESPONSE * time.delta_secs()).exp();
        camera.scale += remaining * ease_out_cubic(blend);
    }

    *target_scale = Some(target);
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn slide_vectors(
    grid: &IsoGrid,
    vertical: &LevelVertical,
    current: Vec2,
    displacement: Vec2,
) -> Vec<Vec2> {
    let mut vectors = vec![
        Vec2::new(displacement.x, 0.0),
        Vec2::new(0.0, displacement.y),
        displacement * 0.5,
    ];

    for direction in [grid.step_u(), grid.step_v()] {
        let direction = direction.normalize_or_zero();
        let amount = displacement.dot(direction);
        if amount.abs() > f32::EPSILON {
            vectors.push(direction * amount);
        }
    }

    for access in &vertical.accesses {
        if access.height_at(grid, current).is_none() {
            continue;
        }
        let direction = (access.to - access.from).normalize_or_zero();
        let amount = displacement.dot(direction);
        if amount.abs() > f32::EPSILON {
            vectors.push(direction * amount);
        }
    }

    vectors
}

fn move_player(
    time: Res<Time>,
    mut player_movement_action: EventReader<MovementAction>,
    grid: Option<Res<IsoGrid>>,
    current: Option<Res<CurrentLevel>>,
    verticals: Query<(&LevelInstance, &LevelVertical)>,
    mut player_query: Query<(&mut Transform, &mut PlayerHeight, &PlayerCollider), With<Player>>,
    obstacle_query: Query<
        (
            &ObstacleCollider,
            Option<&GroundTransform>,
            &GlobalTransform,
        ),
        Without<Player>,
    >,
) {
    const SPEED: f32 = 320.0;

    let movement: Vec2 = player_movement_action
        .read()
        .map(|action| Vec2::from(action))
        .fold(Vec2::ZERO, |x, y| x + y)
        .normalize_or_zero();

    if movement == Vec2::ZERO {
        return;
    }

    let displacement = movement * SPEED * time.delta_secs().min(0.05);
    let vertical = current
        .as_deref()
        .and_then(|current| current.0.as_ref())
        .and_then(|label| {
            verticals
                .iter()
                .find(|(instance, _)| &instance.prefab == label)
        })
        .map(|(_, vertical)| vertical);

    for (mut player_transform, mut player_height, collider) in &mut player_query {
        let feet = player_height.elevation;
        let obstacles: Vec<Vec<Vec2>> = obstacle_query
            .iter()
            .filter(|(obstacle, _, _)| obstacle.overlaps_elevation(feet, PLAYER_BODY_HEIGHT))
            .map(|(obstacle, ground, transform)| match ground {
                Some(ground) => ground.world_corners(obstacle),
                None => obstacle.world_corners(transform),
            })
            .collect();

        let current = player_transform.translation.xy();
        let moved = move_and_slide(current, displacement, collider.radius, &obstacles);

        let mut accepted = moved;
        if let (Some(grid), Some(vertical)) = (grid.as_deref(), vertical) {
            if let Some(sample) = vertical.sample(grid, moved, feet) {
                player_height.elevation = sample.elevation;
                player_height.rank = sample.rank;
            } else {
                let intended = displacement.normalize_or_zero();
                let mut best = None;
                for slide in slide_vectors(grid, vertical, current, displacement) {
                    let candidate = move_and_slide(current, slide, collider.radius, &obstacles);
                    let progress = (candidate - current).dot(intended);
                    if progress <= f32::EPSILON {
                        continue;
                    }
                    let Some(sample) = vertical.sample(grid, candidate, feet) else {
                        continue;
                    };
                    if best
                        .map(|(best_progress, _, _)| progress > best_progress)
                        .unwrap_or(true)
                    {
                        best = Some((progress, candidate, sample));
                    }
                }

                if let Some((_, position, sample)) = best {
                    accepted = position;
                    player_height.elevation = sample.elevation;
                    player_height.rank = sample.rank;
                } else {
                    accepted = current;
                }
            }
        }

        player_transform.translation.x = accepted.x;
        player_transform.translation.y = accepted.y;
    }
}

fn sync_player_visual(
    players: Query<(Entity, &PlayerHeight, &Children), With<Player>>,
    mut roots: Query<&mut Transform, (With<Player>, Without<PlayerVisual>)>,
    mut visuals: Query<&mut Transform, With<PlayerVisual>>,
) {
    for (entity, height, children) in &players {
        if let Ok(mut transform) = roots.get_mut(entity) {
            transform.translation.z = level_z(height.rank, ACTOR_Z_LAYER, transform.translation.y);
        }

        for child in children.iter() {
            if let Ok(mut transform) = visuals.get_mut(*child) {
                transform.translation.y = elevation_screen_offset(height.elevation);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_easing_is_smooth_and_clamped() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        assert!(ease_out_cubic(0.25) > 0.25);
        assert!(ease_out_cubic(0.75) > 0.75);
        assert_eq!(ease_out_cubic(-1.0), 0.0);
        assert_eq!(ease_out_cubic(2.0), 1.0);
    }

    #[test]
    fn stair_edge_movement_offers_a_slide_along_the_access() {
        let grid = IsoGrid::default();
        let from = grid.cell_to_world(Vec2::ZERO);
        let to = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(
            Vec::new(),
            vec![AccessDeclared {
                from,
                to,
                width: grid.step_u().length(),
                from_elevation: 0.0,
                to_elevation: 1.0,
                kind: AccessKind::Stairs,
            }],
        );
        let current = (from + to) * 0.5;
        let run = (to - from).normalize();
        let side = access_side_direction(&grid, from, to);
        let vectors = slide_vectors(&grid, &vertical, current, run * 3.0 + side * 10.0);

        assert!(vectors
            .iter()
            .any(|vector| vector.normalize_or_zero().dot(run).abs() > 0.999));
    }

    #[test]
    fn player_moves_to_level_spawn_request() {
        let mut app = App::new();
        app.init_resource::<PlayerSpawnRequest>()
            .add_systems(Update, apply_player_spawn);

        app.world_mut().spawn((
            Player,
            PlayerHeight::default(),
            Transform::from_xyz(10.0, 10.0, 10.0),
        ));
        app.world_mut().resource_mut::<PlayerSpawnRequest>().0 = Some(Vec2::new(-20.0, 35.0));

        app.update();

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Player>>()
            .single(app.world());

        assert_eq!(transform.translation, Vec3::new(-20.0, 35.0, 10.0));
    }
}
