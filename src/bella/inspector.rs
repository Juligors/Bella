use super::{
    organism::{animal::AnimalMarker, plant::PlantMarker},
    terrain::{tile::TileLayout, TerrainMarker},
    time::{DailyTimer, HourlyTimer, SimTime},
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSet};
use bevy_inspector_egui::{
    bevy_inspector::{ui_for_entity, ui_for_resource, ui_for_world_entities_filtered},
    DefaultInspectorConfigPlugin,
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin, DefaultInspectorConfigPlugin))
            .init_state::<EguiFocusState>()
            .init_state::<EguiVisibleState>()
            .insert_resource(ChosenEntity { entity: None })
            .add_systems(Startup, setup_egui)
            .add_systems(
                Update,
                update_egui_visible_state_based_on_keyboard_input
                    .run_if(in_state(EguiFocusState::IsNotFocused)),
            )
            .add_systems(
                Update,
                (chosen_entity_ui, resources_ui, entities_ui)
                    .run_if(in_state(EguiVisibleState::Yes)),
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

fn setup_egui(mut egui_context: Single<&mut EguiContext, With<PrimaryWindow>>) {
    egui_context
        .get_mut()
        .style_mut(|style| style.visuals.window_shadow = egui::Shadow::NONE);
}

fn chosen_entity_ui(world: &mut World) {
    if let Some(entity) = world.resource::<ChosenEntity>().entity {
        // clear ChosenEntity resource if entity disappeared
        if world.get_entity(entity).is_err() {
            world.resource_mut::<ChosenEntity>().entity = None;
            return;
        }

        let mut egui_context = match world
            .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
            .get_single_mut(world)
        {
            Ok(egui_context) => egui_context.clone(),
            Err(_) => return,
        };

        egui::Window::new("Chosen entity")
            .default_open(true)
            .default_pos((0.0, 70.0))
            .show(egui_context.get_mut(), |ui| {
                if ui.button("Clear chosen entity").clicked() {
                    world.resource_mut::<ChosenEntity>().entity = None;
                }

                ui.separator();

                egui::ScrollArea::both().show(ui, |ui| {
                    ui_for_entity(world, entity, ui);
                });
            });
    }
}

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

fn resources_ui(world: &mut World) {
    let mut egui_context = match world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single_mut(world)
    {
        Ok(egui_context) => egui_context.clone(),
        Err(_) => return,
    };

    egui::Window::new("Resources")
        .default_open(false)
        .default_pos((0.0, 0.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.collapsing("TileLayout", |ui| {
                    ui_for_resource::<TileLayout>(world, ui);
                });

                ui.separator();

                ui.collapsing("Time", |ui| {
                    ui.push_id(0, |ui| {
                        ui_for_resource::<DailyTimer>(world, ui);
                    });

                    ui.separator();

                    ui.push_id(1, |ui| {
                        ui_for_resource::<HourlyTimer>(world, ui);
                    });

                    ui.separator();

                    ui.push_id(2, |ui| {
                        ui_for_resource::<SimTime>(world, ui);
                    });
                });
            });
        });
}

fn entities_ui(world: &mut World) {
    let mut egui_context = match world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single_mut(world)
    {
        Ok(egui_context) => egui_context.clone(),
        Err(_) => return,
    };

    egui::Window::new("Entities")
        .default_open(false)
        .default_pos((0.0, 35.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.collapsing("Terrain", |ui| {
                    ui_for_world_entities_filtered::<With<TerrainMarker>>(world, ui, false);
                });

                ui.separator();

                ui.collapsing("Animals", |ui| {
                    ui_for_world_entities_filtered::<With<AnimalMarker>>(world, ui, false);
                });

                ui.separator();

                ui.collapsing("Plants", |ui| {
                    ui_for_world_entities_filtered::<With<PlantMarker>>(world, ui, false);
                });
            });
        });
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
