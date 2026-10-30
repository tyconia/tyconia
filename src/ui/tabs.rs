use super::*;
use crate::loading::UiAssets;
use crate::ChangeStates;

pub struct TabsPlugin<T: Event + SubStates> {
    event_state: std::marker::PhantomData<T>,
}

impl<T: Event + SubStates> TabsPlugin<T> {
    pub fn new() -> Self {
        Self {
            event_state: std::marker::PhantomData::<T>,
        }
    }
}

impl<T: Event + SubStates + Default> Plugin for TabsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                click_tab::<T>,
                reskin_tabs::<T>,
                handle_set_tab::<T>,
            )
                .run_if(any_with_component::<ChangeStates<T>>),
        );
    }
}

pub fn spawn_tabs<T: Bundle>(
    parent: &mut ChildBuilder,
    tabs: Vec<(String, Handle<Image>, T)>,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    tabs.into_iter().for_each(|(name, icon, tab)| {
        parent
            .spawn(Node {
                flex_grow: 1.,
                ..default()
            })
            .with_children(|parent| {
                spawn_button(
                    ButtonType::LabeledIcon {
                        icon,
                        text: name.into(),
                        font_size: crate::ui::BUTTON_FONT,
                        image_size: Val::Px(crate::ui::UI_SCALE * 3.),
                    },
                    (tab, CustomSkinBehavior),
                    &mut *parent,
                    &fonts,
                    &ui,
                );
            });
    });
}

// handle settings interactions
pub fn click_tab<T: Event + SubStates>(
    mut tab_channel: EventWriter<T>,
    buttons: Query<(&DepressButton, &ChangeStates<T>), Changed<DepressButton>>,
) {
    for (button, tab) in buttons.iter() {
        if button.invoked() {
            tab_channel.send(tab.0.clone());
        }
    }
}

pub fn reskin_tabs<T: Event + SubStates>(
    state: Res<State<T>>,
    mut tabs: Query<(
        &ChangeStates<T>,
        &Interaction,
        &ButtonSkins,
        &mut ImageNode,
        Option<&Children>,
    )>,
    mut labels: Query<&mut TextColor, With<ButtonLabel>>,
) {
    for (tab, interaction, skins, mut image, children) in tabs.iter_mut() {
        let (skin, color) = if tab.0 == *state.get() {
            (skins.active.clone(), Color::WHITE)
        } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
            (skins.hover.clone(), BUTTON_LABEL_COLOR)
        } else {
            (skins.normal.clone(), BUTTON_LABEL_COLOR)
        };

        if image.image != skin {
            image.image = skin;
        }

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut text_color) = labels.get_mut(*child) {
                    if text_color.0 != color {
                        text_color.0 = color;
                    }
                }
            }
        }
    }
}

pub fn handle_set_tab<T: Event + SubStates>(
    mut tab_channel: EventReader<T>,
    mut next_tab: ResMut<NextState<T>>,
) {
    tab_channel.read().for_each(|tab| {
        info!("tab activated: {:?}", tab);
        next_tab.set(tab.clone());
    });
}
