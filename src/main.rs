use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};

mod catalog; 
mod domain; 
mod earth_view;

fn main() {
    // Bevy
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // Primary desktop window created by Bevy/winit.
            primary_window: Some(Window {
                title: "Cozloff Earth Shells".into(),
                resolution: (960, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.012, 0.014, 0.018)))
        .add_plugins(earth_view::EarthViewPlugin)
        .run();
}
