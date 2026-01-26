use bevy::prelude::*;

#[derive(Resource)]
pub struct M8Font {
    pub text_offset_y: u32,
    pub handle: Handle<Image>,
}

fn load_fonts(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(M8Font {
        text_offset_y: 3,
        handle: asset_server.load("font.png"),
    });
}

pub struct M8FontsPlugin;
impl Plugin for M8FontsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_fonts);
    }
}
