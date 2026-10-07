use std::cell::{Cell, UnsafeCell};
use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::JobId;
use rust_state::{Context, Path, Selector};

use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::world::{JobIdentity, Library, Player};

struct ClassTitle<P> {
    player_path: P,
    library: Arc<Library>,
    last_job: Cell<Option<JobId>>,
    title: UnsafeCell<String>,
}

impl<P> ClassTitle<P> {
    fn new(player_path: P, library: Arc<Library>) -> Self {
        Self { player_path, library, last_job: Cell::new(None), title: UnsafeCell::new(String::new()) }
    }
}

impl<P> Selector<ClientState, String> for ClassTitle<P>
where
    P: Path<ClientState, Player, false>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let player = self.player_path.follow(state)?;
        let job = player.get_common().job_id;
        if self.last_job.get() != Some(job) {
            let raw = self.library.get::<JobIdentity>(job).to_string();
            let name = if raw == "1_f_maria" {
                format!("Classe {}", job.0)
            } else {
                raw.split('_')
                    .map(|part| match part {
                        "H" => "Transcendent".to_owned(),
                        "B" => "Baby".to_owned(),
                        _ => {
                            let mut characters = part.chars();
                            characters.next().map(|first| first.to_uppercase().collect::<String>() + &characters.as_str().to_lowercase()).unwrap_or_default()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            // Selectors return a reference to their own persistent text. The UI
            // reads it on the same thread after this update.
            unsafe { *self.title.get() = name };
            self.last_job.set(Some(job));
        }
        unsafe { Some(self.title.as_ref_unchecked()) }
    }
}

struct CharacterStatus<A> {
    player_path: A,
    hp_text: String,
    sp_text: String,
}

impl<A> CharacterStatus<A> {
    fn new(player_path: A) -> Self {
        Self {
            player_path,
            hp_text: String::new(),
            sp_text: String::new(),
        }
    }
}

impl<A> Element<ClientState> for CharacterStatus<A>
where
    A: Path<ClientState, Player, false>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, state: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        if let Some(player) = state.try_get(&self.player_path) {
            let common = player.get_common();
            self.hp_text = format!("HP {}/{}", common.health_points, common.maximum_health_points);
            self.sp_text = format!("SP {}/{}", player.spell_points, player.maximum_spell_points);
        }

        with_single_resolver(resolvers, |resolver| Self::LayoutInfo { area: resolver.with_height(88.0) })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a Context<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let Some(player) = state.try_get(&self.player_path) else { return };
        let area = layout_info.area;
        let avatar_area = Area {
            left: area.left + 8.0,
            top: area.top + 8.0,
            width: 74.0,
            height: area.height - 16.0,
        };

        layout.add_rectangle(
            avatar_area,
            CornerDiameter::uniform(7.0),
            Color::rgba_u8(20, 27, 38, 190),
            Color::rgb_u8(91, 112, 143),
            ShadowPadding::uniform(0.0),
        );

        // Reuse the same composed idle frame that the world renderer uses. It
        // includes the body and the correctly positioned hair attachment.
        // Animation data can be temporarily empty while loading a new map.
        if let Some(animation_data) = player.get_common().animation_data.as_ref() {
            if let Some(frame) = animation_data.animations.first().and_then(|animation| animation.frames.first()) {
                let frame_width = frame.size.x.max(1) as f32;
                let frame_height = frame.size.y.max(1) as f32;
                let scale = ((avatar_area.width - 8.0) / frame_width)
                    .min((avatar_area.height - 8.0) / frame_height)
                    .min(1.5);
                let frame_left = avatar_area.left + (avatar_area.width - frame_width * scale) / 2.0;
                let frame_top = avatar_area.top + (avatar_area.height - frame_height * scale) / 2.0;
                let frame_origin_x = -frame_width / 2.0;
                let frame_origin_y = -frame_height + 0.5;

                for part in &frame.frame_parts {
                    let Some(pair) = animation_data.animation_pair.get(part.animation_index) else { continue };
                    let Some(texture) = pair.sprites.textures.get(part.sprite_number) else { continue };
                    let part_left = frame.offset.x as f32 + part.offset.x as f32 - ((part.size.x - 1) / 2) as f32 - 0.5;
                    let part_top = frame.offset.y as f32 + part.offset.y as f32 - ((part.size.y - 1) / 2) as f32 - 0.5;
                    layout.add_texture(
                        Area {
                            left: frame_left + (part_left - frame_origin_x) * scale,
                            top: frame_top + (part_top - frame_origin_y) * scale,
                            width: part.size.x as f32 * scale,
                            height: part.size.y as f32 * scale,
                        },
                        texture.clone(),
                        part.color,
                        false,
                    );
                }
            }
        }

        let bar_left = area.left + 94.0;
        let bar_width = (area.width - 106.0).max(20.0);
        let common = player.get_common();
        let vitals = [
            (&self.hp_text, common.health_points, common.maximum_health_points, Color::rgb_u8(218, 70, 91), 5.0),
            (&self.sp_text, player.spell_points, player.maximum_spell_points, Color::rgb_u8(61, 142, 226), 43.0),
        ];

        for (text, current, maximum, color, top_offset) in vitals {
            let label_area = Area {
                left: bar_left,
                top: area.top + top_offset,
                width: bar_width,
                height: 20.0,
            };
            layout.add_text(
                label_area,
                text,
                FontSize(14.0),
                Color::WHITE,
                Color::WHITE,
                korangar_interface::layout::alignment::HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
                korangar_interface::layout::alignment::VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );

            let bar_area = Area {
                left: bar_left,
                top: label_area.top + label_area.height + 2.0,
                width: bar_width,
                height: 10.0,
            };
            layout.add_rectangle(bar_area, CornerDiameter::uniform(4.0), Color::rgb_u8(41, 45, 52), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
            let fill_ratio = if maximum == 0 { 0.0 } else { (current as f32 / maximum as f32).clamp(0.0, 1.0) };
            if fill_ratio > 0.0 {
                layout.add_rectangle(
                    Area { width: bar_width * fill_ratio, ..bar_area },
                    CornerDiameter::uniform(4.0),
                    color,
                    Color::TRANSPARENT,
                    ShadowPadding::uniform(0.0),
                );
            }
        }
    }
}

pub struct CharacterOverviewWindow<A, B, C, D> {
    player_name_path: A,
    base_level_path: B,
    job_level_path: C,
    player_path: D,
    library: Arc<Library>,
}

impl<A, B, C, D> CharacterOverviewWindow<A, B, C, D> {
    pub fn new(player_name_path: A, base_level_path: B, job_level_path: C, player_path: D, library: Arc<Library>) -> Self {
        Self {
            player_name_path,
            base_level_path,
            job_level_path,
            player_path,
            library,
        }
    }
}

impl<A, B, C, D> CustomWindow<ClientState> for CharacterOverviewWindow<A, B, C, D>
where
    A: Path<ClientState, String>,
    B: Path<ClientState, usize>,
    C: Path<ClientState, usize>,
    D: Path<ClientState, Player, false> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::CharacterOverview)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: ClassTitle::new(self.player_path, self.library),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 270.0,
            maximum_width: 270.0,
            closable: true,
            elements: (
                CharacterStatus::new(self.player_path),
                fragment! {
                    gaps: 4.0,
                    children: (
                        split! {
                            children: (
                                text! {
                                    text: client_state().localization().name_text(),
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                                text! {
                                    text: self.player_name_path,
                                    color: Color::rgb_u8(255, 144, 13),
                                    horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 3.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                            ),
                        },
                        split! {
                            children: (
                                text! {
                                    text: client_state().localization().base_level_text(),
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                                text! {
                                    text: PartialEqDisplaySelector::new(self.base_level_path),
                                    color: Color::rgb_u8(13, 231, 255),
                                    horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 3.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                            ),
                        },
                        split! {
                            children: (
                                text! {
                                    text: client_state().localization().job_level_text(),
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                                text! {
                                    text: PartialEqDisplaySelector::new(self.job_level_path),
                                    color: Color::rgb_u8(13, 231, 255),
                                    horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 3.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                            ),
                        },
                    ),
                },
                collapsible! {
                    text: "Menu ▾",
                    initially_expanded: false,
                    children: (
                        split! {
                            children: (
                                button! { text: client_state().localization().inventory_button_text(), event: InputEvent::ToggleInventoryWindow },
                                button! { text: client_state().localization().equipment_button_text(), event: InputEvent::ToggleEquipmentWindow },
                            ),
                        },
                        split! {
                            children: (
                                button! { text: client_state().localization().skill_tree_button_text(), event: InputEvent::ToggleSkillTreeWindow },
                                button! { text: client_state().localization().stats_button_text(), event: InputEvent::ToggleStatsWindow },
                            ),
                        },
                        split! {
                            children: (
                                button! { text: client_state().localization().friend_list_button_text(), event: InputEvent::ToggleFriendListWindow },
                                button! { text: client_state().localization().menu_button_text(), event: InputEvent::ToggleMenuWindow },
                            ),
                        },
                    ),
                },
            ),
        }
    }
}
