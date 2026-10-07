use korangar_components::skill_box;
use korangar_interface::element::Element;
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::HotbarSlot;
use rust_state::{ArrayLookupExt, Context, OptionExt, Path, PathExt};

use crate::interface::resource::SkillSource;
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::state::hotbar::{HOTBAR_ROWS, HOTBAR_SLOTS, HOTBAR_SLOTS_PER_ROW, HotbarPathExt};
use crate::state::skills::{LearnableSkill, LearnedSkill, LearnedSkillPath};
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};

pub struct HotbarWindow<A, B> {
    hotbar_path: A,
    skills_path: B,
}

impl<A, B> HotbarWindow<A, B> {
    pub fn new(hotbar_path: A, skills_path: B) -> Self {
        Self { hotbar_path, skills_path }
    }
}

fn hotbar_row<A, B>(hotbar_path: A, skills_path: B, row: usize) -> impl Element<ClientState>
where
    A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]>,
    B: Path<ClientState, Vec<LearnedSkill>>,
{
    use korangar_interface::prelude::*;

    split! {
        gaps: theme().window().gaps(),
        children: std::array::from_fn::<_, HOTBAR_SLOTS_PER_ROW, _>(|index| {
            let slot = row * HOTBAR_SLOTS_PER_ROW + index;
            let learnable_skill_path = hotbar_path.array_index(slot).unwrapped();
            let learned_skill_path = LearnedSkillPath::new(learnable_skill_path, skills_path);

            skill_box! {
                learnable_skill_path,
                learned_skill_path,
                source: SkillSource::Hotbar { slot: HotbarSlot(slot as u16) },
            }
        }),
    }
}

impl<A, B> CustomWindow<ClientState> for HotbarWindow<A, B>
where
    A: Path<ClientState, [Option<LearnableSkill>; HOTBAR_SLOTS]>,
    B: Path<ClientState, Vec<LearnedSkill>>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Hotbar)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: PartialEqDisplaySelector::new(client_state().hotbar().visible_rows()),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            elements: (
                hotbar_row(self.hotbar_path, self.skills_path, 0),
                either! {
                    selector: ComputedSelector::new_default(|state: &ClientState| *client_state().hotbar().visible_rows().follow_safe(state) >= 2),
                    on_true: hotbar_row(self.hotbar_path, self.skills_path, 1),
                    on_false: (),
                },
                either! {
                    selector: ComputedSelector::new_default(|state: &ClientState| *client_state().hotbar().visible_rows().follow_safe(state) >= 3),
                    on_true: hotbar_row(self.hotbar_path, self.skills_path, 2),
                    on_false: (),
                },
                either! {
                    selector: ComputedSelector::new_default(|state: &ClientState| *client_state().hotbar().visible_rows().follow_safe(state) >= 4),
                    on_true: hotbar_row(self.hotbar_path, self.skills_path, 3),
                    on_false: (),
                },
                button! {
                    text: "Fileiras ▾",
                    tooltip: "1: F1-F9 · 2: 1-9 · 3 e 4: configuráveis em Atalhos",
                    event: |state: &Context<ClientState>, queue: &mut EventQueue<ClientState>| {
                        state.update_value_with(client_state().hotbar().visible_rows(), |rows| {
                            *rows = if *rows >= HOTBAR_ROWS { 1 } else { *rows + 1 };
                        });
                        queue.queue(Event::Unfocus);
                    },
                },
                button! {
                    text: "Atalhos",
                    event: InputEvent::ToggleHotkeySettingsWindow,
                },
            )
        }
    }
}
