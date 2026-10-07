use hashbrown::HashMap;
use mlua::Lua;
use ragnarok_packets::ItemId;

use super::{HashMapExt, ItemName, ItemResource, Library, LuaExt, Table, decode_lua_string};
use crate::loaders::GameFileLoader;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub(super) identified_name: ItemName,
    pub(super) unidentified_name: ItemName,
    pub(super) identified_resource: ItemResource,
    pub(super) unidentified_resource: ItemResource,
    identified_description: Vec<String>,
    unidentified_description: Vec<String>,
}

impl ItemInfo {
    pub fn description(&self, identified: bool) -> &[String] {
        if identified {
            &self.identified_description
        } else {
            &self.unidentified_description
        }
    }
}

fn read_description(table: &mlua::Table, field: &str) -> Vec<String> {
    table
        .get::<mlua::Table>(field)
        .ok()
        .map(|lines| {
            lines
                .sequence_values::<mlua::String>()
                .filter_map(Result::ok)
                .map(decode_lua_string)
                .collect()
        })
        .unwrap_or_default()
}

impl Table for ItemInfo {
    type Key<'a> = ItemId;
    type Storage = HashMap<ItemId, Self>;

    fn load(game_file_loader: &GameFileLoader) -> mlua::Result<Self::Storage> {
        let state = Lua::load_from_game_files(game_file_loader, &["data\\luafiles514\\lua files\\datainfo\\iteminfo.lub"])?;

        let globals = state.globals();
        let mut result = HashMap::new();

        if let Ok(table) = globals.get::<mlua::Table>("tbl") {
            for (item_id, item_table) in table.pairs::<u32, mlua::Table>().flatten() {
                let info = ItemInfo {
                    identified_name: ItemName::from_option(item_table.get::<mlua::String>("identifiedDisplayName").ok().map(decode_lua_string)),
                    unidentified_name: ItemName::from_option(item_table.get::<mlua::String>("unidentifiedDisplayName").ok().map(decode_lua_string)),
                    identified_resource: ItemResource::from_option(item_table.get::<mlua::String>("identifiedResourceName").ok().map(decode_lua_string)),
                    unidentified_resource: ItemResource::from_option(item_table.get::<mlua::String>("unidentifiedResourceName").ok().map(decode_lua_string)),
                    identified_description: read_description(&item_table, "identifiedDescriptionName"),
                    unidentified_description: read_description(&item_table, "unidentifiedDescriptionName"),
                };

                result.insert(ItemId(item_id), info);
            }
        }

        Ok(result.compact())
    }

    fn try_get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> Option<&'a Self> {
        library.item_info_table.get(&key)
    }

    fn get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> &'a Self {
        static DEFAULT: ItemInfo = ItemInfo {
            identified_name: ItemName::not_found_value(),
            unidentified_name: ItemName::not_found_value(),
            identified_resource: ItemResource::not_found_value(),
            unidentified_resource: ItemResource::not_found_value(),
            identified_description: Vec::new(),
            unidentified_description: Vec::new(),
        };
        Self::try_get(library, key).unwrap_or(&DEFAULT)
    }
}
