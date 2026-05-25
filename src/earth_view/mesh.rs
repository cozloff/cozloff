use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

// The Earth is rendered as a sphere with a wedge removed.
// This angle controls how wide that missing slice is, in radians.
const OPEN_WEDGE_RADIANS: f32 = 0.82;

// More segments means a smoother sphere, but also more vertices/triangles.
// Longitude wraps around the equator; latitude runs from south pole to north pole.
const LONGITUDE_SEGMENTS: usize = 96;
const LATITUDE_SEGMENTS: usize = 48;

// Build one shell layer as a mesh. Each Earth layer is one of these:
// - an outer curved spherical surface
// - two flat radial cut faces where the wedge is open
pub fn shell_mesh(inner_radius: f32, outer_radius: f32, display_radius: f32) -> Mesh {
    let mut builder = ShellMeshBuilder::new(display_radius);
    builder.add_spherical_band(inner_radius, outer_radius);

    // The wedge is centered around longitude 0. One face is on the positive
    // side of the opening; the other is near TAU, the end of the circle.
    builder.add_cut_face(inner_radius, outer_radius, OPEN_WEDGE_RADIANS * 0.5, false);
    builder.add_cut_face(inner_radius, outer_radius, TAU - OPEN_WEDGE_RADIANS * 0.5, true);

    // Bevy Mesh data is just GPU-ready buffers:
    // - positions: where each vertex is
    // - normals: which direction each vertex faces for lighting
    // - uvs: 2D texture coordinates, included even though we only use colors now
    // - indices: which vertices form each triangle
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, builder.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, builder.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, builder.uvs)
    .with_inserted_indices(Indices::U32(builder.indices))
}

// Small helper that accumulates mesh buffers before turning them into a Bevy Mesh.
struct ShellMeshBuilder {
    display_radius: f32,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl ShellMeshBuilder {
    fn new(display_radius: f32) -> Self {
        Self {
            display_radius,
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_spherical_band(&mut self, _inner_radius: f32, outer_radius: f32) {
        // For the visible outside of each shell, only the outer spherical surface
        // is needed. The exposed layer thickness is shown by the cut faces below.
        self.add_sphere_surface(outer_radius, true);
    }

    fn add_sphere_surface(&mut self, radius: f32, outward: bool) {
        // Skip the wedge opening by starting after half the wedge angle and
        // ending before the other half. The missing longitude range becomes
        // the cutaway.
        let start_lon = OPEN_WEDGE_RADIANS * 0.5;
        let end_lon = TAU - OPEN_WEDGE_RADIANS * 0.5;
        let lon_step = (end_lon - start_lon) / LONGITUDE_SEGMENTS as f32;
        let lat_step = PI / LATITUDE_SEGMENTS as f32;

        // Remember where this surface's vertices start, so the index pass below
        // can refer to the correct vertex IDs.
        let base = self.positions.len() as u32;

        for lat in 0..=LATITUDE_SEGMENTS {
            // phi is latitude angle from -90 degrees to +90 degrees.
            // y is vertical height; ring_radius is the horizontal circle radius
            // at that latitude.
            let phi = -FRAC_PI_2 + lat as f32 * lat_step;
            let y = radius * phi.sin();
            let ring_radius = radius * phi.cos();

            for lon in 0..=LONGITUDE_SEGMENTS {
                // theta is longitude angle around the vertical axis.
                let theta = start_lon + lon as f32 * lon_step;

                // Direction is a unit vector from the center to the sphere
                // surface. Multiplying by radius gives the vertex position.
                let direction = Vec3::new(ring_radius * theta.cos(), y, ring_radius * theta.sin())
                    .normalize_or_zero();

                self.positions
                    .push([direction.x * radius, direction.y * radius, direction.z * radius]);

                // Normals point out for exterior surfaces. The `outward` flag is
                // here so this builder can also support inward-facing surfaces.
                let normal = if outward { direction } else { -direction };
                self.normals.push([normal.x, normal.y, normal.z]);
                self.uvs.push([
                    lon as f32 / LONGITUDE_SEGMENTS as f32,
                    lat as f32 / LATITUDE_SEGMENTS as f32,
                ]);
            }
        }

        let row = LONGITUDE_SEGMENTS as u32 + 1;
        for lat in 0..LATITUDE_SEGMENTS as u32 {
            for lon in 0..LONGITUDE_SEGMENTS as u32 {
                // Four neighboring vertices form one quad on the sphere grid.
                // The renderer only draws triangles, so each quad becomes two.
                let a = base + lat * row + lon;
                let b = a + 1;
                let c = a + row;
                let d = c + 1;

                // Triangle winding order controls which side is considered the
                // front face. The reversed order is needed for inward surfaces.
                if outward {
                    self.indices.extend_from_slice(&[a, c, b, b, c, d]);
                } else {
                    self.indices.extend_from_slice(&[a, b, c, b, d, c]);
                }
            }
        }
    }

    fn add_cut_face(&mut self, inner_radius: f32, outer_radius: f32, theta: f32, flip: bool) {
        let base = self.positions.len() as u32;
        let lat_step = PI / LATITUDE_SEGMENTS as f32;

        // The cut face is a flat wall at a fixed longitude. Its normal points
        // sideways out of the wedge opening.
        let normal = Vec3::new(-theta.sin(), 0.0, theta.cos()) * if flip { -1.0 } else { 1.0 };

        for lat in 0..=LATITUDE_SEGMENTS {
            let phi = -FRAC_PI_2 + lat as f32 * lat_step;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            // Each latitude row stores two vertices: one at the shell's outer
            // radius and one at its inner radius. Connecting those rows creates
            // the rectangular wall that shows the layer thickness.
            for radius in [outer_radius, inner_radius.max(0.001)] {
                let position = Vec3::new(
                    radius * cos_phi * theta.cos(),
                    radius * sin_phi,
                    radius * cos_phi * theta.sin(),
                );

                self.positions.push([position.x, position.y, position.z]);
                self.normals.push([normal.x, normal.y, normal.z]);
                self.uvs.push([
                    radius / self.display_radius,
                    lat as f32 / LATITUDE_SEGMENTS as f32,
                ]);
            }
        }

        for lat in 0..LATITUDE_SEGMENTS as u32 {
            // Connect each pair of latitude rows into two triangles.
            let outer0 = base + lat * 2;
            let inner0 = outer0 + 1;
            let outer1 = outer0 + 2;
            let inner1 = outer0 + 3;

            // The two cut faces sit on opposite sides of the wedge, so one of
            // them needs reversed winding to keep its front face visible.
            if flip {
                self.indices
                    .extend_from_slice(&[outer0, outer1, inner0, inner0, outer1, inner1]);
            } else {
                self.indices
                    .extend_from_slice(&[outer0, inner0, outer1, outer1, inner0, inner1]);
            }
        }
    }
}
