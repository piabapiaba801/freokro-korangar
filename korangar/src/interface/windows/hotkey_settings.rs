use std::cell::UnsafeCell;

use korangar_interface::components::text::Text;
use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Context, PathExt, Selector};

use crate::interface::windows::WindowClass;
use crate::state::hotbar::HotbarPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

pub struct HotkeySettingsWindow;

struct HotkeyTextSelector {
    index: usize,
    text: UnsafeCell<String>,
}

impl HotkeyTextSelector {
    fn new(index: usize) -> Self {
        Self { index, text: UnsafeCell::default() }
    }
}

impl Selector<ClientState, String> for HotkeyTextSelector {
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let hotbar = client_state().hotbar().follow_safe(state);
        let label = if hotbar.capture_slot() == Some(self.index) { "Pressione tecla..." }
            else if hotbar.binding(self.index).is_empty() { "Sem atalho" }
            else { hotbar.binding(self.index) };
        unsafe {
            *self.text.get() = label.to_string();
            Some(self.text.as_ref_unchecked())
        }
    }
}

impl CustomWindow<ClientState> for HotkeySettingsWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::HotkeySettings)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Atalhos da hotbar",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: (
                text! { text: "1: F1-F9 | 2: 1-9 | 3/4: clique em Definir e pressione outra tecla. Esc cancela; Delete limpa." },
                std::array::from_fn::<_, 18, _>(|index| {
                    let binding_text: Text<String, _, _, _, _, _, _, _, _> = text! { text: HotkeyTextSelector::new(index) };
                    split! {
                        gaps: theme().window().gaps(),
                        children: (
                            text! { text: format!("{}.{}", index / 9 + 3, index % 9 + 1) },
                            binding_text,
                            button! {
                                text: "Definir",
                                tooltip: "Pressione a nova tecla. Esc cancela; Delete limpa.",
                                event: move |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                                    state.update_value_with(client_state().hotbar().capture_slot(), move |slot| *slot = Some(index));
                                    queue.queue(Event::Unfocus);
                                },
                            },
                        ),
                    }
                }),
            ),
        }
    }
}
