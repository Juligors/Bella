use bevy::prelude::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::cell::RefCell;

use crate::bella::config::{FloatGeneConfig, IntGeneConfig};

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub struct GenePlugin;

impl Plugin for GenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FloatGene>()
            .register_type::<IntGene>()
            .register_type::<Gene>()
            .register_type::<Allele>()
            .register_type::<AlleleType>();
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct FloatGene {
    pub gene: Gene,
    pub multiplier: f32,
    pub offset: f32,
    phenotype: f32,
}

impl FloatGene {
    pub fn new(gene: Gene, multiplier: f32, offset: f32) -> Self {
        let phenotype = multiplier * (gene.expression_level() + offset).clamp(0.0, 1.0);

        Self {
            gene,
            multiplier,
            offset,
            phenotype,
        }
    }

    pub fn phenotype(&self) -> f32 {
        self.phenotype
    }

    pub fn mixed_with(&self, other: &Self) -> Self {
        // TODO: what about when we mix 2 genes from different species?
        // assert!(self.multiplier == other.multiplier);
        // assert!(self.offset == other.offset);

        FloatGene::new(
            self.gene.cross_with(&other.gene),
            self.multiplier,
            self.offset,
        )
    }
}

impl From<FloatGeneConfig> for FloatGene {
    fn from(value: FloatGeneConfig) -> Self {
        assert!(value.multiplier > 0.0);
        assert!(value.offset >= 0.0);

        Self::new(Gene::new(0.5), value.multiplier, value.offset)
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct IntGene {
    pub gene: Gene,
    pub max_value: u32,
    pub min_value: u32,
    phenotype: u32,
}

impl IntGene {
    pub fn new(gene: Gene, min_value: u32, max_value: u32) -> Self {
        assert!(
            max_value >= min_value,
            "max: {}, min: {}",
            max_value,
            min_value
        );

        let diff = (max_value - min_value) as f32;
        let phenotype = (gene.expression_level() * diff) as u32 + min_value;

        Self {
            gene,
            min_value,
            max_value,
            phenotype,
        }
    }

    pub fn phenotype(&self) -> u32 {
        self.phenotype
    }

    pub fn mixed_with(&self, other: &Self) -> Self {
        // TODO: what about when we mix 2 genes from different species?
        // assert!(self.max_value == other.max_value);
        // assert!(self.min_value == other.min_value);

        Self::new(
            self.gene.cross_with(&other.gene),
            self.min_value,
            self.max_value,
        )
    }
}

impl From<IntGeneConfig> for IntGene {
    fn from(value: IntGeneConfig) -> Self {
        IntGene::new(Gene::new(0.5), value.min_value, value.max_value)
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct Gene {
    pub allele1: Allele,
    pub allele2: Allele,
}

impl Gene {
    pub fn new(gene_starting_value_percentage: f32) -> Self {
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
        }
    }

    pub fn expression_level(&self) -> f32 {
        if self.alleles_have_different_types() {
            if self.allele1.is_dominant() {
                self.allele1.gene_expression_level()
            } else {
                self.allele2.gene_expression_level()
            }
        } else {
            (self.allele1.gene_expression_level() + self.allele2.gene_expression_level()) / 2.0
        }
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

            Gene { allele1, allele2 }
        })
    }

    fn alleles_have_different_types(&self) -> bool {
        self.allele1.is_dominant() && self.allele2.is_recessive()
            || self.allele1.is_recessive() && self.allele2.is_dominant()
    }
}

/// TODO: maska? tzn. "te bity muszą być ustawione na 1, bo jak nie gen nie działa i dajemy wartość 0 i zdychasz elo elo"
/// Mutacja może mieć 2 rezultaty: brak efektu i jakiś efekt. Żeby to osiągnąć moglibyśmy mieć maskę, która "unieaktywnia niektóre obszary", reprezentowałaby ona "trash DNA" and "coding DNA".
/// Poza tym jeśli jest efekt to może on być negatywny jak i pozytywny. To powinno zostać załatwione poprzez naturane działanie genów - albo zwiększamy albo zmiejszamy wartość. Można by pomyśleć o drugiej masce (oryginalny pomysł), która wyłącza cały gen, ale to jest raczej niepotrzebne. Ta nowa maska cześciowo spełnia jej rolę w jakimś tam sensie i chroni też przed negatywnymi efektami mutacji. Pomyśleć czy ona też powinna się zmieniać?
#[derive(Component, Reflect, Debug, Clone)]
pub struct Allele {
    pub allele_type: AlleleType,
    pub bytes: Vec<u8>,
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

/// If there is `Aa` or `aA`, then only `A` works. If there is `AA` or `aa`, we have codominance and both are expressed equally, so we take average.
#[derive(Component, Reflect, Debug, Clone)]
pub enum AlleleType {
    Dominant,
    Recessive,
}
