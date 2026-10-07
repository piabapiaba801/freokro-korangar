use korangar_interface::window::{CustomWindow, Window};
use rust_state::Path;

use crate::interface::windows::WindowClass;
use crate::settings::{InterfaceSettings, InterfaceSettingsCapabilities, InterfaceSettingsCapabilitiesPathExt, InterfaceSettingsPathExt};
use crate::state::theme::InterfaceThemeType;
use crate::state::ClientState;

pub struct SkinWindow<A, B> {
    settings_path: A,
    capabilities_path: B,
}

impl<A, B> SkinWindow<A, B> {
    pub fn new(settings_path: A, capabilities_path: B) -> Self {
        Self { settings_path, capabilities_path }
    }
}

impl<A, B> CustomWindow<ClientState> for SkinWindow<A, B>
where
    A: Path<ClientState, InterfaceSettings>,
    B: Path<ClientState, InterfaceSettingsCapabilities>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Skin)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Skin",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: split! {
                children: (
                    text! { text: "Skin do jogo" },
                    drop_down! {
                        selected: self.settings_path.in_game_theme(),
                        options: self.capabilities_path.in_game_themes(),
                    },
                )
            },
        }
    }
}
