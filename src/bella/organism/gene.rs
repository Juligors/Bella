use bevy::prelude::*;
use bevy_egui::egui::{text::LayoutJob, Color32, TextFormat};
use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct GenePlugin;

impl Plugin for GenePlugin {
    fn build(&self, app: &mut App) {
        app
            // for custom UI
            .register_type_data::<Gene, InspectorEguiImpl>()
            .register_type_data::<Allele, InspectorEguiImpl>();
    }
}

// GeneInterpretation, Gene<T>

// type ReproductionRangeGeneInterpretation = IntGeneInterpretation


#[derive(Reflect, Debug, Clone)]
pub struct GeneInterpretationPercentage{

}

pub trait Interpretation<T> {
    fn interpret(&self)  -> T;
}

#[derive(Reflect, Debug, Clone)]
pub struct Gene<T> {
    pub allele1: Allele,
    pub allele2: Allele,
    pub interpretation: T,
    // gene_expression_multiplier: f32,
    // offset: f32,
}

impl Gene {
    pub fn new(
        gene_expression_multiplier: f32,
        offset: f32,
        gene_starting_value_percentage: f32,
    ) -> Self {
        let byte_value = (gene_starting_value_percentage * 255.0) as u8;

        // TODO: right now it's always 1 Dominant and 1 Recessive. Is that ok? It honestly might be
        Self {
            allele1: Allele {
                allele_type: AlleleType::Dominant,
                bytes: vec![byte_value],
            },
            allele2: Allele {
                allele_type: AlleleType::Recessive,
                bytes: vec![byte_value],
            },
            gene_expression_multiplier,
            offset,
        }
    }

    pub fn phenotype(&self) -> f32 {
        let gene_expression_level = if self.alleles_have_different_types() {
            if self.allele1.is_dominant() {
                self.allele1.gene_expression_level()
            } else {
                self.allele2.gene_expression_level()
            }
        } else {
            (self.allele1.gene_expression_level() + self.allele2.gene_expression_level()) / 2.0
        };

        self.gene_expression_multiplier * (gene_expression_level + self.offset).clamp(0.0, 1.0)
    }

    pub fn cross_with(&self, other: &Gene) -> Self {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            let allele1 = if rng.gen_bool(0.5) {
                self.allele1.clone()
            } else {
                self.allele2.clone()
            };

            let allele2 = if rng.gen_bool(0.5) {
                other.allele1.clone()
            } else {
                other.allele2.clone()
            };

            assert!(self.gene_expression_multiplier == other.gene_expression_multiplier);
            assert!(self.offset == other.offset);

            Gene {
                allele1,
                allele2,
                gene_expression_multiplier: self.gene_expression_multiplier,
                offset: self.offset,
            }
        })
    }

    fn alleles_have_different_types(&self) -> bool {
        self.allele1.is_dominant() && self.allele2.is_recessive()
            || self.allele1.is_recessive() && self.allele2.is_dominant()
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

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.push_id(id.value(), |ui| {
                        ui.label("Gene expression multiplier: ");
                        changed |= env.ui_for_reflect(&mut self.gene_expression_multiplier, ui);
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.push_id(id.value(), |ui| {
                        ui.label("Phenotype: ");
                        changed |= env.ui_for_reflect(&mut self.phenotype(), ui);
                    });
                });
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.push_id(id.value() + 1, |ui| {
                    changed |= env.ui_for_reflect(&mut self.allele1, ui);
                });
                ui.push_id(id.value() + 2, |ui| {
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
            ui.label("Readonly Genetic Information UI, not implemented")
                .changed();
        });
    }
}

/// TODO: maska? tzn. "te bity muszą być ustawione na 1, bo jak nie gen nie działa i dajemy wartość 0 i zdychasz elo elo"
/// Mutacja może mieć 2 rezultaty: brak efektu i jakiś efekt. Żeby to osiągnąć moglibyśmy mieć maskę, która "unieaktywnia niektóre obszary", reprezentowałaby ona "trash DNA" and "coding DNA".
/// Poza tym jeśli jest efekt to może on być negatywny jak i pozytywny. To powinno zostać załatwione poprzez naturane działanie genów - albo zwiększamy albo zmiejszamy wartość. Można by pomyśleć o drugiej masce (oryginalny pomysł), która wyłącza cały gen, ale to jest raczej niepotrzebne. Ta nowa maska cześciowo spełnia jej rolę w jakimś tam sensie i chroni też przed negatywnymi efektami mutacji. Pomyśleć czy ona też powinna się zmieniać?
#[derive(Component, Reflect, Debug, Clone)]
pub struct Allele {
    allele_type: AlleleType,
    bytes: Vec<u8>,
}

///  TODO:
/// TE RZECZY PRZY OKAZJI PRZECHODZENIA NA 1 DNA I VIEWS (CHYBA ŻE Z TEGO REZYGNUJEMY?)
/// - na pewno zaimplemenować podstawowe mutacje: bity, skrócenie i wydłużenie genotypu (skrócenie do 0 bajtów zostawia 1 bajt, tylko go zeruje)
/// - te rzeczy powyżej muszą być na poziomie GeneticInformation, bo np. losowa mutacja zmienia tylko 1 z genów (ale sama zmiana bitów już robiona w Gene)
/// - zobaczyć czy nazwy mają sens
/// - zaimplementować już maskę i ew. zrobić wszędzie 0, czyli maska nie ma żadnego efektu
impl Allele {
    pub fn new(allele_type: AlleleType) -> Self {
        Allele {
            allele_type,
            // TODO: probably change this to some non-hardcoded value? Honestly it might be fine, we would start with 1 "average" species and let more develop
            bytes: vec![127],
        }
    }

    pub fn gene_expression_level(&self) -> f32 {
        let sum: f32 = self.bytes.iter().map(|&x| x as f32).sum();
        let count: f32 = self.bytes.len() as f32;
        let average = sum / count;

        average / u8::MAX as f32
    }

    pub fn mutate(&mut self) {
        // TODO: add rng
        for byte in self.bytes.iter_mut() {
            let index = 7;
            *byte ^= 1 << index;
        }
    }

    pub fn mutate_add_new_blank_byte(&mut self) {
        self.bytes.push(0);
    }

    pub fn is_dominant(&self) -> bool {
        matches!(self.allele_type, AlleleType::Dominant)
    }

    pub fn is_recessive(&self) -> bool {
        matches!(self.allele_type, AlleleType::Recessive)
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
            ui.label("Readonly Gene UI, not implemented").changed();
        });
    }
}

/// If there is `Aa` or `aA`, then only `A` works. If there is `AA` or `aa`, we have codominance and both are expressed equally, so we take average.
#[derive(Component, Reflect, Debug, Clone)]
pub enum AlleleType {
    Dominant,
    Recessive,
}
