use serde::Deserialize;

use crate::domain::earth::{EarthModel, EarthShell, EarthShellCode, EarthShellDomain};
use crate::domain::shell_spec::{DensitySpec, EarthShellSpec, PressureSpec, TemperatureSpec};

const EARTH_SHELLS_JSON: &str = include_str!("../../data/earth/shells.json");
const EARTH_SHELL_SPECS_JSON: &str = include_str!("../../data/earth/shell_specs.json");

#[derive(Debug, Deserialize)]
struct EarthShellRecord {
    code: String,
    name: String,
    domain: String,
    min_depth_km: Option<f64>,
    max_depth_km: Option<f64>,
    min_altitude_km: Option<f64>,
    max_altitude_km: Option<f64>,
    include_in_total_earth: bool,
}

#[derive(Debug, Deserialize)]
struct EarthShellSpecRecord {
    shell_code: String,
    source: String,
    density: Option<DensitySpecRecord>,
    temperature: Option<TemperatureSpecRecord>,
    pressure: Option<PressureSpecRecord>,
}

#[derive(Debug, Deserialize)]
struct DensitySpecRecord {
    average_kg_m3: Option<f64>,
    min_kg_m3: Option<f64>,
    max_kg_m3: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TemperatureSpecRecord {
    average_k: Option<f64>,
    min_k: Option<f64>,
    max_k: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct PressureSpecRecord {
    average_pa: Option<f64>,
    min_pa: Option<f64>,
    max_pa: Option<f64>,
}

// Simple ETL entry point:
// - extract raw JSON records from data/earth/*.json
// - transform stringly JSON values into typed domain enums
// - load the typed records into the EarthModel used by the app
pub fn earth_model() -> EarthModel {
    EarthModel {
        shells: load_shells(),
        specs: load_shell_specs(),
    }
}

fn load_shells() -> Vec<EarthShell> {
    let records = extract_shell_records();

    records
        .into_iter()
        .map(transform_shell_record)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to transform Earth shell JSON into domain structs")
}

fn extract_shell_records() -> Vec<EarthShellRecord> {
    serde_json::from_str(EARTH_SHELLS_JSON).expect("failed to parse data/earth/shells.json")
}

fn load_shell_specs() -> Vec<EarthShellSpec> {
    let records = extract_shell_spec_records();

    records
        .into_iter()
        .map(transform_shell_spec_record)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to transform Earth shell spec JSON into domain structs")
}

fn extract_shell_spec_records() -> Vec<EarthShellSpecRecord> {
    serde_json::from_str(EARTH_SHELL_SPECS_JSON)
        .expect("failed to parse data/earth/shell_specs.json")
}

fn transform_shell_record(record: EarthShellRecord) -> Result<EarthShell, String> {
    Ok(EarthShell {
        code: parse_shell_code(&record.code)?,
        name: record.name,
        domain: parse_shell_domain(&record.domain)?,
        min_depth_km: record.min_depth_km,
        max_depth_km: record.max_depth_km,
        min_altitude_km: record.min_altitude_km,
        max_altitude_km: record.max_altitude_km,
        include_in_total_earth: record.include_in_total_earth,
    })
}

fn transform_shell_spec_record(record: EarthShellSpecRecord) -> Result<EarthShellSpec, String> {
    Ok(EarthShellSpec {
        shell_code: parse_shell_code(&record.shell_code)?,
        source: record.source,
        density: record.density.map(transform_density_spec),
        temperature: record.temperature.map(transform_temperature_spec),
        pressure: record.pressure.map(transform_pressure_spec),
        composition: None,
    })
}

fn transform_density_spec(record: DensitySpecRecord) -> DensitySpec {
    DensitySpec {
        average_kg_m3: record.average_kg_m3,
        min_kg_m3: record.min_kg_m3,
        max_kg_m3: record.max_kg_m3,
    }
}

fn transform_temperature_spec(record: TemperatureSpecRecord) -> TemperatureSpec {
    TemperatureSpec {
        average_k: record.average_k,
        min_k: record.min_k,
        max_k: record.max_k,
    }
}

fn transform_pressure_spec(record: PressureSpecRecord) -> PressureSpec {
    PressureSpec {
        average_pa: record.average_pa,
        min_pa: record.min_pa,
        max_pa: record.max_pa,
    }
}

fn parse_shell_code(code: &str) -> Result<EarthShellCode, String> {
    match code {
        "Exosphere" => Ok(EarthShellCode::Exosphere),
        "Thermosphere" => Ok(EarthShellCode::Thermosphere),
        "Mesosphere" => Ok(EarthShellCode::Mesosphere),
        "Stratosphere" => Ok(EarthShellCode::Stratosphere),
        "Troposphere" => Ok(EarthShellCode::Troposphere),
        "Ocean" => Ok(EarthShellCode::Ocean),
        "ContinentalCrust" => Ok(EarthShellCode::ContinentalCrust),
        "OceanicCrust" => Ok(EarthShellCode::OceanicCrust),
        "Crust" => Ok(EarthShellCode::Crust),
        "LithosphericMantle" => Ok(EarthShellCode::LithosphericMantle),
        "Asthenosphere" => Ok(EarthShellCode::Asthenosphere),
        "MantleTransitionZone" => Ok(EarthShellCode::MantleTransitionZone),
        "LowerMantle" => Ok(EarthShellCode::LowerMantle),
        "CoreMantleBoundary" => Ok(EarthShellCode::CoreMantleBoundary),
        "OuterCore" => Ok(EarthShellCode::OuterCore),
        "InnerCore" => Ok(EarthShellCode::InnerCore),
        unknown => Err(format!("unknown Earth shell code `{unknown}`")),
    }
}

fn parse_shell_domain(domain: &str) -> Result<EarthShellDomain, String> {
    match domain {
        "Atmosphere" => Ok(EarthShellDomain::Atmosphere),
        "Surface" => Ok(EarthShellDomain::Surface),
        "Interior" => Ok(EarthShellDomain::Interior),
        unknown => Err(format!("unknown Earth shell domain `{unknown}`")),
    }
}
