use super::*;
use crate::loading::UiAssets;
use crate::ChangeStates;
use bevy::prelude::*;

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
            OnEnter(T::default()),
            (reskin_active_tab::<T>, handle_set_tab::<T>).chain(),
        )
        .add_systems(
            Update,
            ((
                click_tab::<T>,
                reskin_hover_tab::<T>,
                reskin_active_tab_on::<T>.run_if(on_event::<T>),
                handle_set_tab::<T>,
            )
                .run_if(any_with_component::<ChangeStates<T>>),),
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

pub fn reskin_hover_tab<T: Event + SubStates>(
    mut button: Query<
        (&mut ImageNode, &ButtonSkins, &Interaction),
        (With<ChangeStates<T>>, Changed<Interaction>),
    >,
) {
    for (mut image, skin, interaction) in button.iter_mut() {
        if image.image != skin.active {
            match *interaction {
                Interaction::Hovered => {
                    image.image = skin.hover.clone();
                }
                Interaction::None => {
                    image.image = skin.normal.clone();
                }
                _ => {}
            }
        }
    }
}

pub fn reskin_active_tab<T: Event + SubStates>(
    tab: Res<State<T>>,
    mut tab_channel: EventWriter<T>,
) {
    info!("active tab is {:?}", tab);
    tab_channel.send(tab.clone());
}

pub fn reskin_active_tab_on<T: Event + SubStates>(
    mut cmd: Commands,
    mut active_tab: EventReader<T>,
    mut skins: Query<(
        Option<&Children>,
        &mut ImageNode,
        &ButtonSkins,
        &ChangeStates<T>,
    )>,
) {
    active_tab.read().last().map(|active_tab| {
        skins
            .iter_mut()
            .for_each(|(children, mut current_skin, button_skins, state)| {
                if state.0 == *active_tab {
                    current_skin.image = button_skins.active.clone();
                    children.map(|children| {
                        cmd.entity(children.first().unwrap().clone())
                            .insert(TextColor(Color::srgba(1., 1., 1., 1.0)));
                    });
                } else {
                    current_skin.image = button_skins.normal.clone();

                    children.map(|children| {
                        cmd.entity(children.first().unwrap().clone())
                            .insert(TextColor(Color::srgba(0.356, 0.333, 0.333, 1.0)));
                    });
                }
            });
    });
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
