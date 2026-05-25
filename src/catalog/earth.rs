use crate::domain::earth::{EarthModel, EarthShell, EarthShellCode, EarthShellDomain};

pub const INNER_CORE: EarthShell = EarthShell {
    code: EarthShellCode::InnerCore,
    name: "Inner Core",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(5155.0),
    max_depth_km: Some(6371.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};

pub const OUTER_CORE: EarthShell = EarthShell {
    code: EarthShellCode::OuterCore,
    name: "Outer Core",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(2885.0),
    max_depth_km: Some(5155.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};

pub const LOWER_MANTLE: EarthShell = EarthShell {
    code: EarthShellCode::LowerMantle,
    name: "Lower Mantle",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(700.0),
    max_depth_km: Some(2885.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};

pub const LITHOSPHERIC_MANTLE: EarthShell = EarthShell {
    code: EarthShellCode::LithosphericMantle,
    name: "Lithospheric Mantle",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(35.0),
    max_depth_km: Some(700.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};

pub const OCEANIC_CRUST: EarthShell = EarthShell {
    code: EarthShellCode::OceanicCrust,
    name: "Oceanic Crust",
    domain: EarthShellDomain::Surface,
    min_depth_km: Some(4.0),
    max_depth_km: Some(14.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: false,
};

pub const OCEAN: EarthShell = EarthShell {
    code: EarthShellCode::Ocean,
    name: "Ocean",
    domain: EarthShellDomain::Surface,
    min_depth_km: Some(0.0),
    max_depth_km: Some(4.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};

pub const CRUST: EarthShell = EarthShell {
    code: EarthShellCode::Crust,
    name: "Crust",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(0.0),
    max_depth_km: Some(35.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: true,
};


pub const TROPOSPHERE: EarthShell = EarthShell {
    code: EarthShellCode::Troposphere,
    name: "Troposphere",
    domain: EarthShellDomain::Atmosphere,
    min_depth_km: None,
    max_depth_km: None,
    min_altitude_km: Some(0.0),
    max_altitude_km: Some(11.0),
    include_in_total_earth: true,
};

pub const CRUST_AGGREGATE: EarthShell = EarthShell {
    code: EarthShellCode::Crust,
    name: "Crust",
    domain: EarthShellDomain::Interior,
    min_depth_km: Some(0.0),
    max_depth_km: Some(35.0),
    min_altitude_km: None,
    max_altitude_km: None,
    include_in_total_earth: false,
};

pub const STRATOSPHERE: EarthShell = EarthShell {
    code: EarthShellCode::Stratosphere,
    name: "Stratosphere",
    domain: EarthShellDomain::Atmosphere,
    min_depth_km: None,
    max_depth_km: None,
    min_altitude_km: Some(20.0),
    max_altitude_km: Some(50.0),
    include_in_total_earth: true,
};

pub const MESOSPHERE: EarthShell = EarthShell {
    code: EarthShellCode::Mesosphere,
    name: "Mesosphere",
    domain: EarthShellDomain::Atmosphere,
    min_depth_km: None,
    max_depth_km: None,
    min_altitude_km: Some(50.0),
    max_altitude_km: Some(85.0),
    include_in_total_earth: true,
};

pub const THERMOSPHERE: EarthShell = EarthShell {
    code: EarthShellCode::Thermosphere,
    name: "Thermosphere",
    domain: EarthShellDomain::Atmosphere,
    min_depth_km: None,
    max_depth_km: None,
    min_altitude_km: Some(85.0),
    max_altitude_km: Some(690.0),
    include_in_total_earth: true,
};

pub const EXOSPHERE: EarthShell = EarthShell {
    code: EarthShellCode::Exosphere,
    name: "Exosphere",
    domain: EarthShellDomain::Atmosphere,
    min_depth_km: None,
    max_depth_km: None,
    min_altitude_km: Some(690.0),
    max_altitude_km: Some(10000.0),
    include_in_total_earth: true,
};

pub fn earth_model() -> EarthModel {
    EarthModel {
        shells: vec![
            INNER_CORE,
            OUTER_CORE,
            LOWER_MANTLE,
            LITHOSPHERIC_MANTLE,
            OCEANIC_CRUST,
            OCEAN,
            CRUST,
            TROPOSPHERE,
            STRATOSPHERE,
            MESOSPHERE,
            THERMOSPHERE,
            EXOSPHERE,
        ],
        compositions: Vec::new(),
        densities: Vec::new(),
    }
}
