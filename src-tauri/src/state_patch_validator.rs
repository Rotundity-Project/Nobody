use serde_json::{Map, Value};

pub fn default_key_field(table: &str) -> &'static str {
    match table {
        "characters" => "character_id",
        "map_nodes" => "location_id",
        "map_edges" => "from_id",
        "techniques" => "technique_id",
        "inventory_items" => "item_id",
        "factions" => "faction_id",
        "story_state" => "chapter_index",
        "world_facts" => "fact_id",
        _ => "id",
    }
}

pub fn validate_patch_row(table: &str, row: &Map<String, Value>) -> Result<(), String> {
    match table {
        "characters" => {
            require_non_empty_str(row, "character_id", table)?;
            require_non_empty_str(row, "name", table)?;
            require_non_empty_str(row, "role", table)?;
        }
        "map_nodes" => {
            require_non_empty_str(row, "location_id", table)?;
            require_non_empty_str(row, "name", table)?;
            require_non_empty_str(row, "description", table)?;
        }
        "map_edges" => {
            require_non_empty_str(row, "from_id", table)?;
            require_non_empty_str(row, "to_id", table)?;
            require_u64_like(row, "travel_days", table)?;
        }
        "techniques" => {
            require_non_empty_str(row, "technique_id", table)?;
            require_non_empty_str(row, "name", table)?;
            require_non_empty_str(row, "description", table)?;
        }
        "inventory_items" => {
            require_non_empty_str(row, "item_id", table)?;
            require_non_empty_str(row, "owner_character_id", table)?;
            require_non_empty_str(row, "name", table)?;
            require_non_empty_str(row, "item_type", table)?;
        }
        "factions" => {
            require_non_empty_str(row, "faction_id", table)?;
            require_non_empty_str(row, "name", table)?;
        }
        "story_state" => {
            require_u64_like(row, "chapter_index", table)?;
            require_non_empty_str(row, "chapter_goal", table)?;
            require_non_empty_str(row, "current_arc", table)?;
        }
        "world_facts" => {
            require_non_empty_str(row, "fact_id", table)?;
            require_non_empty_str(row, "subject", table)?;
            require_non_empty_str(row, "predicate", table)?;
            require_non_empty_str(row, "object", table)?;
        }
        _ => return Err(format!("unknown table in state_patch: {}", table)),
    }
    Ok(())
}

fn require_non_empty_str(row: &Map<String, Value>, key: &str, table: &str) -> Result<(), String> {
    let value = row
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    if value.is_empty() {
        return Err(format!("table {} missing required string field {}", table, key));
    }
    Ok(())
}

fn require_u64_like(row: &Map<String, Value>, key: &str, table: &str) -> Result<(), String> {
    if row.get(key).and_then(Value::as_u64).is_none() {
        return Err(format!("table {} missing required number field {}", table, key));
    }
    Ok(())
}
