use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::catalog::astral::EARTH_RADIUS_KM;
use crate::catalog::earth::earth_model;
use crate::domain::earth::{EarthShell, EarthShellCode, EarthShellDomain};

mod material; 
mod mesh;

const DISPLAY_RADIUS: f32 = 2.4;

pub struct EarthViewPlugin;

// Render-ready version of an EarthShell. 
// Converted from kilometers into Bevy world-space radii.
#[derive(Debug, Clone, Copy)]
struct ShellBand {
    code: EarthShellCode,
    inner_radius: f32,
    outer_radius: f32,
}

impl Plugin for EarthViewPlugin {
    fn build(&self, app: &mut App) {
        // `Startup` system 
        //  - initial camera, light, and Earth shell entities.
        app.add_systems(Startup, setup_earth_view);
    }
}

fn setup_earth_view(
    // Commands ECS mutation API. Used to spawn entities,
    // add components, and request world changes from inside a system.
    mut commands: Commands,
    // Assets<T> asset storage resource. For adding a Mesh / Material
    // getting a handle; and storing in an entity store w/o owning heavy GPU data.
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_camera(&mut commands);
    spawn_light(&mut commands);
    spawn_shells(&mut commands, meshes.as_mut(), materials.as_mut());
}

fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        // Camera3d marks entity as a 3D camera -> Transform places it.
        Camera3d::default(),
        // The minimal Bevy feature set doesn't enable tonemapping LUTs,
        // avoids requiring extra feature.
        Tonemapping::None,
        Transform::from_xyz(0.0, 1.8, 6.4).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_light(commands: &mut Commands) {
    commands.spawn((
        // Parallel light rays from one direction to simulate sunlight.
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.8, -0.3)),
    ));
}

fn spawn_shells(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for band in solid_shell_bands() {
        commands.spawn((
            // Mesh3d and MeshMaterial3d are components. Together they tell Bevy:
            // render this entity using this mesh asset and this material asset.
            Mesh3d(meshes.add(mesh::shell_mesh(
                band.inner_radius,
                band.outer_radius,
                DISPLAY_RADIUS,
            ))),
            MeshMaterial3d(materials.add(material::shell_material(band.code))),
            // The mesh wedge is built around the +X axis. This rotation turns
            // that cutout toward the camera, otherwise the view looks like a
            // plain exterior ball.
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.28, -FRAC_PI_2, 0.0)),
        ));
    }
}

fn solid_shell_bands() -> Vec<ShellBand> {
    // Convert catalog EarthShells into renderable radial bands.
    // Atmosphere is skipped because this view is a solid cutaway ball, not a
    // huge transparent atmosphere visualization.
    let mut bands = earth_model()
        .shells
        .into_iter()
        .filter(|shell| shell.include_in_total_earth)
        .filter(|shell| shell.domain != EarthShellDomain::Atmosphere)
        .filter_map(|shell| {
            let inner_radius = shell_inner_radius(&shell) * DISPLAY_RADIUS;
            let outer_radius = shell_outer_radius(&shell) * DISPLAY_RADIUS;

            (outer_radius > inner_radius).then_some(ShellBand {
                code: shell.code,
                inner_radius,
                outer_radius,
            })
        })
        .collect::<Vec<_>>();

    // Sort from the center outward. If two shells share an outer boundary, the
    // thinner surface shell comes after the deeper shell and can trim it.
    bands.sort_by(|a, b| {
        a.inner_radius
            .partial_cmp(&b.inner_radius)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.outer_radius
                    .partial_cmp(&b.outer_radius)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    // Some catalog layers overlap, such as Crust and Ocean both touching the
    // outer Earth radius. Overlapping geometry causes z-fighting and flicker.
    // This pass trims the previous deeper band wherever a newer outer band
    // starts, producing one clean solid layer at each radius.
    let mut normalized = Vec::<ShellBand>::new();
    for band in bands {
        while let Some(previous) = normalized.last_mut() {
            if previous.outer_radius > band.inner_radius {
                previous.outer_radius = band.inner_radius;
            }

            if previous.outer_radius <= previous.inner_radius {
                normalized.pop();
                continue;
            }

            break;
        }

        normalized.push(band);
    }

    normalized
}

fn shell_inner_radius(shell: &EarthShell) -> f32 {
    // Depth-based shells count down from the surface, so larger max_depth means
    // a smaller inner radius. Altitude-based shells count outward from Earth.
    match (shell.max_depth_km, shell.min_altitude_km) {
        (Some(max_depth), _) => ((EARTH_RADIUS_KM - max_depth as f32) / EARTH_RADIUS_KM).max(0.0),
        (_, Some(min_altitude)) => 1.0 + min_altitude as f32 / EARTH_RADIUS_KM,
        _ => 0.0,
    }
}

fn shell_outer_radius(shell: &EarthShell) -> f32 {
    // A shell's outer boundary is either its minimum depth below the surface or
    // its maximum altitude above the surface.
    match (shell.min_depth_km, shell.max_altitude_km) {
        (Some(min_depth), _) => ((EARTH_RADIUS_KM - min_depth as f32) / EARTH_RADIUS_KM).max(0.0),
        (_, Some(max_altitude)) => 1.0 + max_altitude as f32 / EARTH_RADIUS_KM,
        _ => 1.0,
    }
}
