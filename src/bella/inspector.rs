use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSet};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin, DefaultInspectorConfigPlugin))
            .init_state::<EguiFocusState>()
            .add_systems(Update, inspector_ui)
            .add_systems(
                PostUpdate,
                check_egui_wants_focus.after(EguiSet::InitContexts),
            );
    }
}

fn inspector_ui(world: &mut World, mut disabled: Local<bool>) {
    let space_pressed = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::Space);

    if space_pressed {
        *disabled = !*disabled;
    }

    if *disabled {
        return;
    }

    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    // the usual `ResourceInspector` code
    egui::Window::new("Resource Inspector").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            // bevy_inspector_egui::bevy_inspector::ui_for_resource::<Configuration>(world, ui);

            ui.separator();
            ui.label("Press space to toggle");
        });
    });
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EguiFocusState {
    #[default]
    IsNotFocused,
    IsFocused,
}

pub fn check_egui_wants_focus(
    mut egui_contexts: bevy_egui::EguiContexts,
    window_entity: Single<Entity, With<Window>>,
    mut egui_wants_focus_before: Local<bool>,
    mut egui_wants_focus_now: Local<bool>,
    mut next_state: ResMut<NextState<EguiFocusState>>,
) {
    *egui_wants_focus_before = *egui_wants_focus_now;

    if let Some(ctx) = egui_contexts.try_ctx_for_entity_mut(window_entity.into_inner()) {
        *egui_wants_focus_now = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
        *egui_wants_focus_now |= ctx.is_pointer_over_area();
    } else {
        *egui_wants_focus_now = false;
    };

    if *egui_wants_focus_before || *egui_wants_focus_now {
        next_state.set(EguiFocusState::IsFocused);
    } else {
        next_state.set(EguiFocusState::IsNotFocused);
    }
}
