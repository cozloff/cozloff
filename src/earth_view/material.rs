use bevy::prelude::*;

use crate::domain::earth::EarthShellCode;

// Converts a earth shell code into a Bevy PBR material.
pub fn shell_material(code: EarthShellCode) -> StandardMaterial {
    let color = shell_color(code);
    
    StandardMaterial {
        base_color: color, 
        unlit: true, 
        perceptual_roughness: 0.78,
        metallic: 0.0,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    }
}

// Lookup table from Earth shell data to render colors.
fn shell_color(code: EarthShellCode) -> Color {
    match code {
        // Core layers use hot colors so the deepest layers read immediately.
        EarthShellCode::InnerCore => Color::srgb(1.0, 0.86, 0.28),
        EarthShellCode::OuterCore => Color::srgb(0.95, 0.38, 0.18),

        // Mantle/crust layers use earth tones.
        EarthShellCode::LowerMantle => Color::srgb(0.66, 0.27, 0.20),
        EarthShellCode::LithosphericMantle => Color::srgb(0.48, 0.32, 0.24),
        EarthShellCode::Crust | EarthShellCode::OceanicCrust | EarthShellCode::ContinentalCrust => {
            Color::srgb(0.38, 0.58, 0.34)
        }
        EarthShellCode::Ocean => Color::srgb(0.12, 0.45, 0.78),
        
        // Atmosphere colors are defined even though the current solid ball view
        // filters atmosphere shells out before rendering.
        EarthShellCode::Troposphere
        | EarthShellCode::Stratosphere
        | EarthShellCode::Mesosphere
        | EarthShellCode::Thermosphere
        | EarthShellCode::Exosphere => Color::srgb(0.42, 0.74, 1.0),
        _ => Color::srgb(0.70, 0.70, 0.70),
    }
}
