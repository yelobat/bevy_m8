//! This is responsible for loading the necessary font files.

use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState},
};

use crate::M8LoadingState;

/// The assets used by the M8.
#[derive(AssetCollection, Resource)]
pub struct M8Assets {
    #[asset(path = "font.png")]
    pub font_small: Handle<Image>,
}

/// This plugin provides asset loading capabilities.
pub struct M8AssetsPlugin;

impl Plugin for M8AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<M8LoadingState>().add_loading_state(
            LoadingState::new(M8LoadingState::Loading)
                .continue_to_state(M8LoadingState::Running)
                .load_collection::<M8Assets>(),
        );
    }
}
