use bevy::prelude::*;

pub struct UiFacadePlugin;

impl Plugin for UiFacadePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EguiFocusState>()
            .init_state::<EguiVisibleState>()
            .insert_resource(ChosenEntity { entity: None });
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EguiVisibleState {
    #[default]
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EguiFocusState {
    #[default]
    IsNotFocused,
    IsFocused,
}

#[derive(Resource)]
pub struct ChosenEntity {
    pub entity: Option<Entity>,
}

#[cfg(not(feature = "bella_headless"))]
pub fn choose_entity_observer(
    click: Trigger<Pointer<Click>>,
    mut chosen_entity: ResMut<ChosenEntity>,
    egui_focus_state: Res<State<EguiFocusState>>,
) {
    if matches!(**egui_focus_state, EguiFocusState::IsFocused) {
        return;
    }

    if click.button != PointerButton::Primary {
        return;
    }

    chosen_entity.entity = Some(click.target);
}

#[cfg(feature = "bella_headless")]
pub fn choose_entity_observer(click: Trigger<Pointer<Click>>) {}
