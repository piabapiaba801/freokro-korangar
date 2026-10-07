use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::prelude::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::window::{CustomWindow, Window};
use korangar_networking::{InventoryItem, InventoryItemDetails};
use ragnarok_packets::ItemOptions;
use rust_state::Context;

use super::WindowClass;
use crate::graphics::{Color, CornerDiameter, ShadowPadding, Texture};
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;
use crate::world::{ItemCardMetadata, ResourceMetadata};

static OPTION_TEMPLATES: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    ron::from_str(include_str!("random_option_templates.ron")).unwrap_or_default()
});

#[derive(Clone, Debug)]
pub struct ItemDetailsData {
    name: String,
    item_id: u32,
    identified: bool,
    icon: Option<Arc<Texture>>,
    description: Vec<String>,
    facts: Vec<(String, String)>,
    cards: Vec<ItemCardMetadata>,
    options: Vec<String>,
}

fn type_name(item_type: u8) -> &'static str {
    match item_type {
        0 => "Consumível",
        2 => "Utilizável",
        3 => "Diversos",
        4 => "Arma",
        5 => "Armadura",
        6 => "Carta",
        7 => "Ovo de mascote",
        8 => "Equipamento de mascote",
        10 => "Munição",
        11 => "Consumível",
        18 => "Item de loja",
        _ => "Item",
    }
}

fn option_text(option: &ItemOptions) -> Option<String> {
    if option.index == 0 {
        return None;
    }
    let value = option.value.to_string();
    Some(match OPTION_TEMPLATES.get(&option.index) {
        Some(template) => template.replace("%%", "\u{0}").replace("%d", &value).replace('\0', "%"),
        None => format!("Opção #{} · valor {}", option.index, value),
    })
}

impl ItemDetailsData {
    fn from_metadata(metadata: &ResourceMetadata, identified: bool) -> Self {
        let mut facts = vec![("Tipo".to_owned(), type_name(metadata.item_type).to_owned())];
        if let Some(weight) = metadata.weight {
            facts.push(("Peso".to_owned(), format!("{}.{:01}", weight / 10, weight % 10)));
        }
        Self {
            name: metadata.name.clone(),
            item_id: metadata.item_id,
            identified,
            icon: metadata.texture.clone(),
            description: metadata.description.clone(),
            facts,
            cards: metadata.cards.clone(),
            options: Vec::new(),
        }
    }

    pub fn inventory(item: &InventoryItem<ResourceMetadata>) -> Self {
        let mut details = Self::from_metadata(&item.metadata, item.is_identified());
        match &item.details {
            InventoryItemDetails::Regular { amount, .. } => {
                details.facts.push(("Quantidade".to_owned(), amount.to_string()));
            }
            InventoryItemDetails::Equippable { refinement_level, enchantment_level, option_data, .. } => {
                if *refinement_level > 0 {
                    details.name = format!("+{} {}", refinement_level, details.name);
                    details.facts.push(("Aprimoramento".to_owned(), format!("+{}", refinement_level)));
                }
                if *enchantment_level > 0 {
                    details.facts.push(("Grau de encantamento".to_owned(), enchantment_level.to_string()));
                }
                details.options.extend(option_data.iter().filter_map(option_text));
            }
        }
        details
    }

    pub fn shop(metadata: &ResourceMetadata, quantity: Option<u32>, price: u32) -> Self {
        let mut details = Self::from_metadata(metadata, true);
        if let Some(quantity) = quantity {
            details.facts.push(("Quantidade".to_owned(), quantity.to_string()));
        }
        details.facts.push(("Preço".to_owned(), format!("{} z", price)));
        details
    }
}

#[derive(Clone, Copy)]
enum LineKind {
    Section,
    Text,
    Card,
    Fact,
    Rule,
}

struct DetailLine {
    text: String,
    kind: LineKind,
}

impl DetailLine {
    fn height(&self) -> f32 {
        match self.kind {
            LineKind::Section => 30.0,
            LineKind::Card => 30.0,
            LineKind::Rule => 12.0,
            LineKind::Text | LineKind::Fact => 19.0,
        }
    }
}

fn visible_text(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'^' && index + 7 <= bytes.len() && bytes[index + 1..index + 7].iter().all(u8::is_ascii_hexdigit) {
            index += 7;
        } else {
            let character = text[index..].chars().next().unwrap();
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

fn last_color_code(text: &str, current: Option<&str>) -> Option<String> {
    let bytes = text.as_bytes();
    let mut color = current.map(str::to_owned);
    let mut index = 0;
    while index + 7 <= bytes.len() {
        if bytes[index] == b'^' && bytes[index + 1..index + 7].iter().all(u8::is_ascii_hexdigit) {
            let code = &text[index..index + 7];
            color = if code.eq_ignore_ascii_case("^000000") { None } else { Some(code.to_owned()) };
            index += 7;
        } else {
            index += text[index..].chars().next().unwrap().len_utf8();
        }
    }
    color
}

fn append_text(lines: &mut Vec<DetailLine>, text: &str, kind: LineKind) {
    let visible = visible_text(text);
    if visible.trim().is_empty() {
        return;
    }

    // Preserve Ragnarok color codes: the font layouter already renders them.
    // A 400-pixel window at 13 px fits roughly 53 Latin characters.
    for raw_line in text.lines() {
        let visible_line = visible_text(raw_line);
        if !visible_line.trim().is_empty() && visible_line.trim().chars().all(|character| character == '_') {
            lines.push(DetailLine { text: String::new(), kind: LineKind::Rule });
            continue;
        }
        let mut line = String::new();
        let mut color: Option<String> = None;
        let mut visible_length = 0;
        for word in raw_line.split_whitespace() {
            let word_length = visible_text(word).chars().count();
            if visible_length > 0 && visible_length + word_length + 1 > 53 {
                lines.push(DetailLine { text: std::mem::take(&mut line), kind });
                if let Some(active_color) = &color {
                    line.push_str(active_color);
                }
                visible_length = 0;
            }
            if visible_length > 0 {
                line.push(' ');
                visible_length += 1;
            }
            line.push_str(word);
            visible_length += word_length;
            color = last_color_code(word, color.as_deref());
        }
        if visible_length > 0 {
            lines.push(DetailLine { text: line, kind });
        }
    }
}

struct ItemDetailsBody {
    details: ItemDetailsData,
    lines: Vec<DetailLine>,
    height: f32,
    footer: String,
}

impl ItemDetailsBody {
    fn new(details: ItemDetailsData) -> Self {
        let mut lines = Vec::new();
        lines.push(DetailLine { text: "Descrição e efeitos".to_owned(), kind: LineKind::Section });
        if details.description.is_empty() {
            append_text(&mut lines, "Descrição não disponível nos arquivos do cliente.", LineKind::Text);
        } else {
            for description in &details.description {
                append_text(&mut lines, description, LineKind::Text);
            }
        }

        lines.push(DetailLine { text: "Dados do item".to_owned(), kind: LineKind::Section });
        for (label, value) in &details.facts {
            append_text(&mut lines, &format!("{label}: {value}"), LineKind::Fact);
        }

        if !details.cards.is_empty() {
            lines.push(DetailLine { text: "Cartas equipadas".to_owned(), kind: LineKind::Section });
            for card in &details.cards {
                lines.push(DetailLine { text: card.name.clone(), kind: LineKind::Card });
                for description in &card.description {
                    append_text(&mut lines, description, LineKind::Text);
                }
            }
        }

        if !details.options.is_empty() {
            lines.push(DetailLine { text: "Opções adicionais".to_owned(), kind: LineKind::Section });
            for option in &details.options {
                append_text(&mut lines, option, LineKind::Text);
            }
        }

        let height = 78.0 + lines.iter().map(DetailLine::height).sum::<f32>() + 36.0;
        let footer = format!("ID {}", details.item_id);
        Self { details, lines, height, footer }
    }
}

impl Element<ClientState> for ItemDetailsBody {
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(&mut self, _: &Context<ClientState>, _: ElementStoreMut, resolvers: &mut dyn Resolvers<ClientState>) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo { area: resolver.with_height(self.height) })
    }

    fn lay_out<'a>(&'a self, _: &'a Context<ClientState>, _: ElementStore<'a>, layout_info: &'a Self::LayoutInfo, layout: &mut WindowLayout<'a, ClientState>) {
        let area = layout_info.area;
        layout.add_rectangle(area, CornerDiameter::uniform(4.0), Color::rgb_u8(36, 43, 40), Color::rgb_u8(8, 13, 9), ShadowPadding::uniform(2.0));
        let header = Area { height: 70.0, ..area };
        layout.add_rectangle(header, CornerDiameter::uniform(4.0), Color::rgb_u8(56, 68, 60), Color::rgb_u8(82, 97, 81), ShadowPadding::uniform(1.0));
        let icon_area = Area { left: area.left + 12.0, top: area.top + 12.0, width: 46.0, height: 46.0 };
        layout.add_rectangle(icon_area, CornerDiameter::uniform(3.0), Color::rgb_u8(53, 64, 56), Color::rgb_u8(97, 114, 99), ShadowPadding::uniform(1.0));
        if let Some(icon) = &self.details.icon {
            layout.add_texture(icon_area, icon.clone(), Color::WHITE, false);
        } else {
            layout.add_text(icon_area, "?", FontSize(22.0), Color::rgb_u8(193, 211, 184), Color::WHITE,
                HorizontalAlignment::Center { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
        }
        let heading = Area { left: area.left + 69.0, top: area.top + 13.0, width: area.width - 80.0, height: 27.0 };
        layout.add_text(heading, &self.details.name, FontSize(15.0), Color::rgb_u8(231, 222, 180), Color::WHITE,
            HorizontalAlignment::Left { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
        let status = if self.details.identified { "Item identificado" } else { "Item não identificado" };
        layout.add_text(Area { top: heading.top + 28.0, height: 20.0, ..heading }, status, FontSize(12.0),
            Color::rgb_u8(176, 189, 175), Color::WHITE, HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);

        let mut top = area.top + 78.0;
        let mut card_index = 0;
        for line in &self.lines {
            let height = line.height();
            let line_area = Area { left: area.left + 13.0, top, width: area.width - 26.0, height };
            if matches!(line.kind, LineKind::Rule) {
                layout.add_rectangle(Area { top: top + 5.0, height: 1.0, ..line_area }, CornerDiameter::uniform(0.0),
                    Color::rgb_u8(82, 98, 82), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
                top += height;
                continue;
            }
            let (font_size, color) = match line.kind {
                LineKind::Section => {
                    layout.add_rectangle(Area { top: top + 3.0, height: height - 6.0, ..line_area }, CornerDiameter::uniform(0.0),
                        Color::rgb_u8(51, 67, 55), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
                    layout.add_rectangle(Area { top: top + 3.0, width: 3.0, height: height - 6.0, ..line_area }, CornerDiameter::uniform(0.0),
                        Color::rgb_u8(131, 173, 120), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
                    (FontSize(12.0), Color::rgb_u8(186, 221, 170))
                }
                LineKind::Card => (FontSize(13.0), Color::rgb_u8(203, 181, 230)),
                LineKind::Fact => (FontSize(12.0), Color::rgb_u8(226, 232, 222)),
                LineKind::Text => (FontSize(13.0), Color::rgb_u8(226, 232, 222)),
                LineKind::Rule => unreachable!(),
            };
            let mut text_area = if matches!(line.kind, LineKind::Section) { Area { left: line_area.left + 8.0, width: line_area.width - 8.0, ..line_area } } else { line_area };
            if matches!(line.kind, LineKind::Card) {
                let icon_area = Area { left: line_area.left + 2.0, top: top + 3.0, width: 24.0, height: 24.0 };
                layout.add_rectangle(icon_area, CornerDiameter::uniform(2.0), Color::rgb_u8(53, 64, 56),
                    Color::rgb_u8(97, 114, 99), ShadowPadding::uniform(1.0));
                if let Some(icon) = self.details.cards.get(card_index).and_then(|card| card.texture.as_ref()) {
                    layout.add_texture(icon_area, icon.clone(), Color::WHITE, false);
                }
                card_index += 1;
                text_area.left += 32.0;
                text_area.width -= 32.0;
            }
            layout.add_text(text_area, &line.text, font_size, color, Color::WHITE,
                HorizontalAlignment::Left { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
            top += height;
        }

        let footer = Area { left: area.left, top: top + 4.0, width: area.width, height: 26.0 };
        layout.add_rectangle(footer, CornerDiameter::uniform(0.0), Color::rgb_u8(45, 56, 48), Color::TRANSPARENT, ShadowPadding::uniform(0.0));
        layout.add_text(Area { left: footer.left + 12.0, width: footer.width - 24.0, ..footer }, &self.footer, FontSize(11.0), Color::rgb_u8(170, 185, 170), Color::WHITE,
            HorizontalAlignment::Right { offset: 0.0, border: 0.0 }, VerticalAlignment::Center { offset: 0.0 }, OverflowBehavior::Shrink);
    }
}

pub struct ItemDetailsWindow {
    details: ItemDetailsData,
}

impl ItemDetailsWindow {
    pub fn new(details: ItemDetailsData) -> Self {
        Self { details }
    }
}

impl CustomWindow<ClientState> for ItemDetailsWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::ItemDetails)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        window! {
            title: "Informações do item",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 400.0,
            maximum_width: 400.0,
            minimum_height: 180.0,
            maximum_height: 600.0,
            closable: true,
            resizable: true,
            elements: scroll_view! { children: ItemDetailsBody::new(self.details) },
        }
    }
}
