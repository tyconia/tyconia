use crate::loading::*;
use crate::ui::*;

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                cycle_state::<CheckboxState>.run_if(any_with_component::<CheckboxState>),
                cycle_state::<CheckboxTristate>.run_if(any_with_component::<CheckboxTristate>),
            ),
        );
    }
}

pub fn cycle_state<C: CycleState>(
    mut cmd: Commands,
    ui: Res<UiAssets>,
    mut checkbox: Query<(&mut C, &DepressButton, &Children), Changed<DepressButton>>,
) {
    for (mut checkbox, depress, children) in checkbox.iter_mut() {
        if depress.invoked() {
            checkbox.cycle_state();
            checkbox.refresh_icon(&mut cmd, &*children, &ui);
        }
    }
}

/// Represents checkbox states
#[derive(Debug, Component, Clone, Copy)]
pub enum CheckboxTristate {
    Active,
    Inactive,
    Disabled,
}

impl CycleState for CheckboxTristate {
    fn next(&self) -> Self {
        match *self {
            Self::Active => Self::Disabled,
            Self::Inactive => Self::Active,
            Self::Disabled => Self::Inactive,
        }
    }

    fn icon(&self, ui: &Res<UiAssets>) -> Option<Handle<Image>> {
        match *self {
            Self::Active => Some(ui.check.clone()),
            Self::Inactive => None,
            Self::Disabled => Some(ui.cross.clone()),
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub enum CheckboxState {
    Active,
    Inactive,
}

impl CycleState for CheckboxState {
    fn next(&self) -> Self {
        match *self {
            Self::Active => Self::Inactive,
            Self::Inactive => Self::Active,
        }
    }

    fn icon(&self, ui: &Res<UiAssets>) -> Option<Handle<Image>> {
        match *self {
            Self::Active => Some(ui.check.clone()),
            Self::Inactive => None,
        }
    }
}

pub trait CycleState: Clone + Copy + Component {
    fn cycle_state(&mut self) {
        *self = self.next();
    }

    fn refresh_icon(&self, cmd: &mut Commands, children: &Children, ui: &Res<UiAssets>) {
        let child = children.first().unwrap();

        let mut child_commands = cmd.entity(*child);
        let icon = self.icon(ui);

        match icon {
            Some(image) => {
                child_commands.insert((ImageNode { image, ..default() },));
            }

            None => {
                child_commands.remove::<ImageNode>();
            }
        }
    }

    fn next(&self) -> Self;

    fn icon(&self, ui: &Res<UiAssets>) -> Option<Handle<Image>>;
}

/// Textured checkbox.
///
/// # Arguments
///
/// * `commands` - commands used to spawn the button
///
/// # Usage
/// just use it bro
pub fn spawn_checkbox<'a, 'b, C: CycleState>(
    state: C,
    components: impl Bundle,
    cmd: &'a mut ChildBuilder<'b>,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
) {
    cmd.spawn((
        Node {
            aspect_ratio: Some(1.),
            height: Val::Px(UI_SCALE * 4.),
            width: Val::Px(UI_SCALE * 4.),
            margin: UiRect::all(Val::Px(UI_SCALE)),
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ))
    .with_children(|parent| {
        super::spawn_button(
            super::ButtonType::Icon {
                image: state.icon(&ui),
                image_size: Val::Px(super::UI_SCALE * 3.),
            },
            (state, components),
            parent,
            fonts,
            ui,
        );
    });
}
