use bevy::utils::hashbrown::HashMap;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{Distribution, Normal, StandardNormal, Uniform, WeightedIndex};
use serde::Deserialize;
use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum BooleanDistribution {
    Chance { chance: f32 },
}

impl BooleanDistribution {
    pub fn happened(&self) -> bool {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                BooleanDistribution::Chance { chance } => rng.gen_bool(*chance as f64),
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum DiscreteDistribution {
    Range {
        min: i32,
        max: i32,
    },
    Choice {
        choices: Vec<i32>,
    },
    WeightedChoice {
        choices: Vec<i32>,
        weights: Vec<f32>,
    },
}

impl DiscreteDistribution {
    pub fn sample(&self) -> i32 {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                DiscreteDistribution::Range { min, max } => rng.gen_range(*min..*max + 1),
                DiscreteDistribution::Choice { choices } => {
                    choices[rng.gen_range(0..choices.len())]
                }
                DiscreteDistribution::WeightedChoice {
                    choices,
                    weights,
                } => {
                    let dist = WeightedIndex::new(weights).expect("Invalid weights");

                    choices[rng.sample(dist)]
                }
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ContinuousDistribution {
    Normal {
        mean: f32,
        std: f32,
        min: Option<f32>,
        max: Option<f32>,
    },
    Uniform {
        min: f32,
        max: f32,
    },
}

impl ContinuousDistribution {
    pub fn sample(&self) -> f32 {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();

            match self {
                ContinuousDistribution::Normal {
                    mean,
                    std,
                    min,
                    max,
                } => {
                    let result = rng.sample(
                        Normal::new(*mean, *std).expect("Failed to create standard distribution"),
                    );

                    result.clamp(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
                }
                ContinuousDistribution::Uniform { min, max } => {
                    rng.sample(Uniform::new(*min, *max))
                }
            }
        })
    }
}
