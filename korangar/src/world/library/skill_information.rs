use std::sync::LazyLock;

use hashbrown::HashMap;
use mlua::Lua;
use ragnarok_packets::{SkillId, SkillLevel};

use super::{HashMapExt, Library, Table, decode_lua_string};
use crate::loaders::GameFileLoader;
use crate::state::skills::SkillAcquisition;
use crate::world::library::LuaExt;

static NOT_FOUND_ENTRY: LazyLock<SkillListInformation> = LazyLock::new(|| SkillListInformation {
    file_name: "notfound".to_owned(),
    name: "notfound".to_owned(),
    description: "Skill description unavailable".to_owned(),
    maximum_level: SkillLevel(100),
    can_select_level: false,
    // To make it unskillable.
    acquisition: SkillAcquisition::Quest,
});

pub struct SkillListInformation {
    pub file_name: String,
    pub name: String,
    pub description: String,
    pub maximum_level: SkillLevel,
    pub can_select_level: bool,
    pub acquisition: SkillAcquisition,
}

fn plain_skill_description(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'^'
            && index + 7 <= bytes.len()
            && bytes[index + 1..index + 7].iter().all(u8::is_ascii_hexdigit)
        {
            index += 7;
        } else {
            let character = text[index..].chars().next().unwrap();
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

impl Table for SkillListInformation {
    type Key<'a> = SkillId;
    type Storage = HashMap<SkillId, Self>;

    fn load(game_file_loader: &GameFileLoader) -> mlua::Result<Self::Storage> {
        let state = Lua::load_from_game_files(game_file_loader, &[
            // Needed to get the `JOBID` table.
            "data\\luafiles514\\lua files\\skillinfoz\\jobinheritlist.lub",
            // Needed to get the `SKID` table.
            "data\\luafiles514\\lua files\\skillinfoz\\skillid.lub",
            "data\\luafiles514\\lua files\\skillinfoz\\skillinfolist.lub",
            "data\\luafiles514\\lua files\\skillinfoz\\skilldescript.lub",
        ])?;

        let globals = state.globals();
        let skill_info_list = globals.get::<mlua::Table>("SKILL_INFO_LIST")?;
        let skill_descriptions = globals.get::<mlua::Table>("SKILL_DESCRIPT")?;

        let mut result = HashMap::new();

        for (skill_id, table) in skill_info_list.pairs::<u16, mlua::Table>().flatten() {
            let file_name = table.get(1)?;
            let name = table.get::<mlua::String>("SkillName").map(decode_lua_string)?;
            let description = skill_descriptions
                .get::<mlua::Table>(skill_id)
                .ok()
                .map(|lines| {
                    lines
                        .sequence_values::<mlua::String>()
                        .filter_map(Result::ok)
                        .map(decode_lua_string)
                        .map(|line| plain_skill_description(&line))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| name.clone());
            let maximum_level = table.get("MaxLv")?;
            let can_select_level = table.get("bSeperateLv")?;
            let acquisition = match table.get::<String>("Type").ok().as_deref() {
                Some("Quest") => SkillAcquisition::Quest,
                Some("Soul") => SkillAcquisition::SoulLink,
                None => SkillAcquisition::Job,
                Some(unknown) => panic!("unknown skill type {}", unknown),
            };

            result.insert(SkillId(skill_id), SkillListInformation {
                file_name,
                name,
                description,
                maximum_level: SkillLevel(maximum_level),
                can_select_level,
                acquisition,
            });
        }

        Ok(result.compact())
    }

    fn try_get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> Option<&'a Self>
    where
        Self: Sized,
    {
        library.skill_information_table.get(&key)
    }

    fn get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> &'a Self
    where
        Self: Sized,
    {
        Self::try_get(library, key).unwrap_or(&*NOT_FOUND_ENTRY)
    }
}
