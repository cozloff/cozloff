use crate::domain::composition::CompositionProfile;
use crate::domain::earth::EarthShellCode;

#[derive(Debug, Clone)]
pub struct EarthShellSpec {
    pub shell_code: EarthShellCode,
    pub source: String,

    pub density: Option<DensitySpec>,
    pub temperature: Option<TemperatureSpec>,
    pub pressure: Option<PressureSpec>,
    pub composition: Option<CompositionProfile>,
}

#[derive(Debug, Clone)]
pub struct DensitySpec {
    pub average_kg_m3: Option<f64>,
    pub min_kg_m3: Option<f64>,
    pub max_kg_m3: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TemperatureSpec {
    pub average_k: Option<f64>,
    pub min_k: Option<f64>,
    pub max_k: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PressureSpec {
    pub average_pa: Option<f64>,
    pub min_pa: Option<f64>,
    pub max_pa: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct VisualSpec {
    pub color_hint: Option<[f32; 3]>,
    pub emissive_strength: Option<f32>,
}
