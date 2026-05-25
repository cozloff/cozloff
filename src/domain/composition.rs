use std::collections::HashMap;
use crate::domain::element::{Element, ElementFraction};
use crate::domain::earth::EarthShellCode;

// EarthShell
// + EarthShellSpec::DensitySpec
// + CompositionProfile
//     ↓
// AtomDistribution

#[derive(Debug, Clone)]
pub struct CompositionProfile {
    pub shell_code: EarthShellCode,
    pub source: String,
    pub fractions: Vec<ElementFraction>,
}

// volume = geometric shell volume
// mass = density * volume
// element mass = mass * element_mass_fraction
// moles = element mass / atomic_mass
// atoms = moles * Avogadro constant

#[derive(Debug, Clone)]
pub struct AtomDistribution {
    pub shell_code: EarthShellCode,
    pub volume_km3: f64,
    pub mass_kg: f64,
    pub atoms_by_element: HashMap<Element, f64>,
    pub atom_fraction_by_element: HashMap<Element, f64>,
}

#[derive(Debug, Clone)]
pub struct EarthBodyAggregate {
    pub body_code: i32,
    pub total_volume_km3: f64,
    pub total_mass_kg: f64,
    pub atoms_by_element: HashMap<Element, f64>,
    pub atom_fraction_by_element: HashMap<Element, f64>,
}
