use super::shell_spec::EarthShellSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum EarthShellCode {
    // Atmosphere: 3990xx
    Exosphere = 399001,
    Thermosphere = 399002,
    Mesosphere = 399003,
    Stratosphere = 399004,
    Troposphere = 399005,

    // Surface / near-surface: 3991xx
    Ocean = 399101,
    ContinentalCrust = 399102,
    OceanicCrust = 399103,

    // Interior: 3992xx
    Crust = 399201,
    LithosphericMantle = 399202,
    Asthenosphere = 399203,
    MantleTransitionZone = 399204,
    LowerMantle = 399205,
    CoreMantleBoundary = 399206,
    OuterCore = 399207,
    InnerCore = 399208,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EarthShellDomain {
    Atmosphere,
    Surface,
    Interior,
}

#[derive(Debug, Clone)]
pub struct EarthShell {
    pub code: EarthShellCode,
    pub name: String,
    pub domain: EarthShellDomain,

    // Internal shells use depth below surface
    pub min_depth_km: Option<f64>,
    pub max_depth_km: Option<f64>,

    // Atmosphere shells use altitude above surface
    pub min_altitude_km: Option<f64>,
    pub max_altitude_km: Option<f64>,

    pub include_in_total_earth: bool,
}

pub struct EarthModel {
    pub shells: Vec<EarthShell>,
    pub specs: Vec<EarthShellSpec>,
}

impl EarthModel {
    pub fn shell(&self, code: EarthShellCode) -> Option<&EarthShell> {
        self.shells.iter().find(|s| s.code == code)
    }

    pub fn spec(&self, code: EarthShellCode) -> Option<&EarthShellSpec> {
        self.specs.iter().find(|spec| spec.shell_code == code)
    }
}
