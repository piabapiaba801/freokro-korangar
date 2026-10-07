use korangar_interface::element::StateElement;
use korangar_networking::NetworkingSystem;
use ragnarok_packets::handler::PacketCallback;
use ragnarok_packets::{HotbarSlot, HotbarTab, HotkeyData, HotkeyType};
use rust_state::RustState;

use crate::state::skills::LearnableSkill;

pub const HOTBAR_ROWS: usize = 4;
pub const HOTBAR_SLOTS_PER_ROW: usize = 9;
pub const HOTBAR_SLOTS: usize = HOTBAR_ROWS * HOTBAR_SLOTS_PER_ROW;

#[derive(RustState, StateElement)]
pub struct Hotbar {
    skills: [Option<LearnableSkill>; HOTBAR_SLOTS],
    visible_rows: usize,
    bindings: [String; 18],
    capture_slot: Option<usize>,
}

impl Default for Hotbar {
    fn default() -> Self {
        let bindings = std::fs::read_to_string("client/hotkey_settings.ron")
            .ok()
            .and_then(|data| ron::from_str::<[String; 18]>(&data).ok())
            .unwrap_or_else(|| std::array::from_fn(|_| String::new()));
        Self { skills: std::array::from_fn(|_| None), visible_rows: 1, bindings, capture_slot: None }
    }
}

impl Hotbar {
    pub fn capture_slot(&self) -> Option<usize> {
        self.capture_slot
    }

    pub fn set_binding(&mut self, index: usize, binding: String) {
        if !binding.is_empty() {
            for (other_index, other_binding) in self.bindings.iter_mut().enumerate() {
                if other_index != index && *other_binding == binding {
                    other_binding.clear();
                }
            }
        }
        self.bindings[index] = binding;
        self.capture_slot = None;
        if let Ok(data) = ron::ser::to_string_pretty(&self.bindings, ron::ser::PrettyConfig::new()) {
            let _ = std::fs::write("client/hotkey_settings.ron", data);
        }
    }

    pub fn cancel_capture(&mut self) {
        self.capture_slot = None;
    }

    pub fn binding(&self, index: usize) -> &str {
        &self.bindings[index]
    }

    /// Set the slot without notifying the map server.
    pub fn set_slot(&mut self, slot: HotbarSlot, skill: LearnableSkill) {
        self.skills[slot.0 as usize] = Some(skill);
    }

    /// Update the slot and notify the map server.
    pub fn update_slot<Callback>(&mut self, networking_system: &mut NetworkingSystem<Callback>, slot: HotbarSlot, skill: LearnableSkill)
    where
        Callback: PacketCallback + Send,
    {
        let _ = networking_system.set_hotkey_data(HotbarTab(0), slot, HotkeyData {
            hotkey_type: HotkeyType::Skill,
            item_or_skill_id: skill.skill_id.0 as u32,
            quantity_or_skill_level: skill.maximum_level.0,
        });

        self.skills[slot.0 as usize] = Some(skill);
    }

    /// Swap two slots in the hotbar and notify the map server.
    pub fn swap_slot<Callback>(
        &mut self,
        networking_system: &mut NetworkingSystem<Callback>,
        source_slot: HotbarSlot,
        destination_slot: HotbarSlot,
    ) where
        Callback: PacketCallback + Send,
    {
        if source_slot != destination_slot {
            let first = self.skills[source_slot.0 as usize].take();
            let second = self.skills[destination_slot.0 as usize].take();

            let first_data = first
                .as_ref()
                .map(|skill| HotkeyData {
                    hotkey_type: HotkeyType::Skill,
                    item_or_skill_id: skill.skill_id.0 as u32,
                    quantity_or_skill_level: skill.maximum_level.0,
                })
                .unwrap_or(HotkeyData::UNBOUND);

            let second_data = second
                .as_ref()
                .map(|skill| HotkeyData {
                    hotkey_type: HotkeyType::Skill,
                    item_or_skill_id: skill.skill_id.0 as u32,
                    quantity_or_skill_level: skill.maximum_level.0,
                })
                .unwrap_or(HotkeyData::UNBOUND);

            let _ = networking_system.set_hotkey_data(HotbarTab(0), destination_slot, first_data);
            let _ = networking_system.set_hotkey_data(HotbarTab(0), source_slot, second_data);

            self.skills[source_slot.0 as usize] = second;
            self.skills[destination_slot.0 as usize] = first;
        }
    }

    /// Clear the slot without notifying the map server.
    pub fn unset_slot(&mut self, slot: HotbarSlot) {
        self.skills[slot.0 as usize] = None;
    }

    /// Clear the slot and notify the map server.
    pub fn clear_slot<Callback>(&mut self, networking_system: &mut NetworkingSystem<Callback>, slot: HotbarSlot)
    where
        Callback: PacketCallback + Send,
    {
        let _ = networking_system.set_hotkey_data(HotbarTab(0), slot, HotkeyData::UNBOUND);

        self.skills[slot.0 as usize] = None;
    }

    pub fn get_skill_in_slot(&self, slot: HotbarSlot) -> &Option<LearnableSkill> {
        &self.skills[slot.0 as usize]
    }
}
