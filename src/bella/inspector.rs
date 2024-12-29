use bevy::{ecs::system::SystemState, prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSet};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

use super::{
    organism::{
        animal::AnimalMarker,
        plant::{PlantAssets, PlantMarker},
    },
    terrain::{tile::TileLayout, TerrainMarker},
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin, DefaultInspectorConfigPlugin))
            .init_state::<EguiFocusState>()
            .init_state::<EguiVisibleState>()
            .add_systems(
                Update,
                update_egui_visible_state_based_on_keyboard_input.run_if(in_state(EguiFocusState::IsNotFocused)),
            )
            .add_systems(
                Update,
                (resources_ui, entities_ui).run_if(in_state(EguiVisibleState::Yes)),
            )
            .add_systems(
                PostUpdate,
                update_egui_focus_state.after(EguiSet::InitContexts),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EguiVisibleState {
    #[default]
    Yes,
    No,
}

fn update_egui_visible_state_based_on_keyboard_input(
    current_state: Res<State<EguiVisibleState>>,
    mut next_state: ResMut<NextState<EguiVisibleState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(match **current_state {
            EguiVisibleState::Yes => EguiVisibleState::No,
            EguiVisibleState::No => EguiVisibleState::Yes,
        });
    }
}

fn resources_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single_mut(world)
        .clone();

    egui::Window::new("Resources")
        .default_open(false)
        .default_pos((0.0, 0.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.collapsing("TileLayout", |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_resource::<TileLayout>(world, ui);
                });
            });
        });
}

fn entities_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single_mut(world)
        .clone();

    egui::Window::new("Entities")
        .default_open(false)
        .default_pos((0.0, 35.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.collapsing("Terrain", |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_world_entities_filtered::<
                        With<TerrainMarker>,
                    >(world, ui, false);
                });

                ui.separator();

                ui.collapsing("Animals", |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_world_entities_filtered::<
                        With<AnimalMarker>,
                    >(world, ui, false);
                });

                ui.separator();

                ui.collapsing("Plants", |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_world_entities_filtered::<
                        With<PlantMarker>,
                    >(world, ui, false);
                });
            });
        });
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EguiFocusState {
    #[default]
    IsNotFocused,
    IsFocused,
}

pub fn update_egui_focus_state(
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
