use super::{
    organism::{
        animal::AnimalMarker,
        gene::{Allele, Gene, UnsignedFloatGene, UnsignedIntGene},
        plant::PlantMarker,
    },
    terrain::{tile::TileLayout, TerrainMarker},
    time::{DayTimer, TimeUnitTimer, SimulationTime},
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::egui::{text::LayoutJob, Color32, TextFormat};
use bevy_egui::{
    egui::{self},
    EguiContext, EguiPlugin, EguiSet,
};
use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
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
            // for custom UI for Genes
            .register_type_data::<UnsignedFloatGene, InspectorEguiImpl>()
            .register_type_data::<UnsignedIntGene, InspectorEguiImpl>()
            .register_type_data::<Gene, InspectorEguiImpl>()
            .register_type_data::<Allele, InspectorEguiImpl>()
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
            .default_width(700.0)
            .show(egui_context.get_mut(), |ui| {
                if ui.button("Clear chosen entity").clicked() {
                    world.resource_mut::<ChosenEntity>().entity = None;
                }

                ui.separator();

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
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
                        ui_for_resource::<DayTimer>(world, ui);
                    });

                    ui.separator();

                    ui.push_id(1, |ui| {
                        ui_for_resource::<TimeUnitTimer>(world, ui);
                    });

                    ui.separator();

                    ui.push_id(2, |ui| {
                        ui_for_resource::<SimulationTime>(world, ui);
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

impl InspectorPrimitive for UnsignedFloatGene {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_egui::egui::Id,
        mut env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Multiplier: ");
                    changed |=
                        env.ui_for_reflect_with_options(&mut self.multiplier, ui, id, options);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Offset: ");
                    changed |= env.ui_for_reflect_with_options(&mut self.offset, ui, id, options);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Phenotype: ");
                    changed |=
                        env.ui_for_reflect_with_options(&mut self.phenotype(), ui, id, options);
                });
            });

            changed |= env.ui_for_reflect_with_options(&mut self.gene, ui, id, options);
        });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_egui::egui::Ui,
        _: &dyn std::any::Any,
        _: bevy_egui::egui::Id,
        _: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        ui.add_enabled_ui(false, |ui| {
            ui.label("Readonly UnsignedFloatGene UI, not implemented")
                .changed();
        });
    }
}

impl InspectorPrimitive for UnsignedIntGene {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_egui::egui::Id,
        mut env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Max_value: ");
                    changed |=
                        env.ui_for_reflect_with_options(&mut self.max_value, ui, id, options);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Min_value: ");
                    changed |=
                        env.ui_for_reflect_with_options(&mut self.min_value, ui, id, options);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Phenotype: ");
                    changed |=
                        env.ui_for_reflect_with_options(&mut self.phenotype(), ui, id, options);
                });
            });

            changed |= env.ui_for_reflect_with_options(&mut self.gene, ui, id, options);
        });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_egui::egui::Ui,
        _: &dyn std::any::Any,
        _: bevy_egui::egui::Id,
        _: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        ui.add_enabled_ui(false, |ui| {
            ui.label("Readonly UnsignedIntGene UI, not implemented")
                .changed();
        });
    }
}

impl InspectorPrimitive for Gene {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: &dyn std::any::Any,
        id: bevy_egui::egui::Id,
        mut env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.push_id(id.value().wrapping_add(100), |ui| {
                    changed |= env.ui_for_reflect(&mut self.allele1, ui);
                });
                ui.push_id(id.value().wrapping_add(200), |ui| {
                    changed |= env.ui_for_reflect(&mut self.allele2, ui);
                });
            });
        });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_egui::egui::Ui,
        _: &dyn std::any::Any,
        _: bevy_egui::egui::Id,
        _: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        ui.add_enabled_ui(false, |ui| {
            ui.label("Readonly Gene UI, not implemented!").changed();
        });
    }
}

impl InspectorPrimitive for Allele {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_egui::egui::Id,
        mut env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            ui.push_id(id.value(), |ui| {
                changed |= env.ui_for_reflect(&mut self.allele_type, ui);
            });

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        for byte in self.bytes.iter() {
                            let mut job = LayoutJob::default();
                            for bit in format!("{:0>8b}", byte).chars() {
                                let color = if bit == '1' {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                };
                                job.append(
                                    &bit.to_string(),
                                    0.0,
                                    TextFormat { color, ..default() },
                                );
                            }
                            job.append("\n", 0.0, TextFormat::default());

                            // make border invisible
                            // let style = ui.style_mut();
                            // style.visuals.widgets.noninteractive.bg_stroke = Stroke {
                            //     color: Color32::TRANSPARENT,
                            //     ..Default::default()
                            // };
                            ui.group(|ui| {
                                changed |= ui.label(job).changed();
                            });
                        }
                    });

                    ui.push_id(id.value(), |ui| {
                        ui.set_max_width(150.0);
                        changed |=
                            env.ui_for_reflect_with_options(&mut self.bytes, ui, id, options);
                    });
                });
            });
        });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_egui::egui::Ui,
        _: &dyn std::any::Any,
        _: bevy_egui::egui::Id,
        _: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        ui.add_enabled_ui(false, |ui| {
            ui.label("Readonly Allele UI, not implemented!").changed();
        });
    }
}
