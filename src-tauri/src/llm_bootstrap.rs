use crate::game_state::GameState;
use crate::state_patch_validator::validate_patch_row;
use serde_json::Value;

pub fn build_bootstrap_prompt(game_state: &GameState, reference_78: &str, supplement: &str) -> String {
    format!(
        "You are a world-state bootstrap generator for a xianxia game. Build tables first, do not write prose.\n\
Use reference docs 78 + supplement as hard constraints.\n\
[ref_78]\n{}\n\
[supplement]\n{}\n\
[player]\nname={} location={} realm={} age={}\n\
[world_counts]\nlocations={} factions={} techniques={}\n\
Return strict JSON only:\n\
{{\
  \"characters\": [],\
  \"map_nodes\": [],\
  \"map_edges\": [],\
  \"techniques\": [],\
  \"inventory_items\": [],\
  \"factions\": [],\
  \"story_state\": [],\
  \"world_facts\": []\
}}\n",
        clip(reference_78, 2200),
        clip(supplement, 2200),
        game_state.player.name,
        game_state.player.location,
        game_state.player.stats.cultivation_realm.name,
        game_state.player.stats.lifespan.current_age,
        game_state.script.world_setting.locations.len(),
        game_state.script.world_setting.factions.len(),
        game_state.script.world_setting.techniques.len(),
    )
}

pub fn build_bootstrap_repair_prompt(
    game_state: &GameState,
    reference_78: &str,
    supplement: &str,
    error: &str,
    previous_output: &str,
) -> String {
    format!(
        "{}\n\
[RETRY]\n\
Your previous bootstrap JSON failed validation.\n\
Validation error: {}\n\
Previous output snippet:\n{}\n\
Return a full corrected JSON object only. Do not omit any top-level table key.",
        build_bootstrap_prompt(game_state, reference_78, supplement),
        error,
        clip(previous_output, 1200),
    )
}

pub fn validate_bootstrap_payload(value: &Value, game_state: &GameState) -> Result<(), String> {
    let Some(obj) = value.as_object() else {
        return Err("bootstrap payload must be a JSON object".to_string());
    };
    let required_tables = [
        "characters",
        "map_nodes",
        "map_edges",
        "techniques",
        "inventory_items",
        "factions",
        "story_state",
        "world_facts",
    ];
    for table in required_tables {
        if !obj.contains_key(table) {
            return Err(format!("bootstrap missing top-level table {}", table));
        }
        if !obj.get(table).map(Value::is_array).unwrap_or(false) {
            return Err(format!("bootstrap table {} must be an array", table));
        }
    }

    let chars = obj
        .get("characters")
        .and_then(Value::as_array)
        .ok_or_else(|| "bootstrap characters must be array".to_string())?;
    if chars.is_empty() {
        return Err("bootstrap characters cannot be empty".to_string());
    }
    for row in chars {
        let Some(map) = row.as_object() else {
            return Err("bootstrap characters row must be object".to_string());
        };
        validate_patch_row("characters", map)?;
    }
    let has_player = chars.iter().any(|row| {
        row.get("role").and_then(Value::as_str) == Some("player")
            || row.get("character_id").and_then(Value::as_str) == Some(&game_state.player.id)
    });
    if !has_player {
        return Err("bootstrap characters must include player row".to_string());
    }

    for table in [
        "map_nodes",
        "map_edges",
        "techniques",
        "inventory_items",
        "factions",
        "story_state",
        "world_facts",
    ] {
        let rows = obj
            .get(table)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("bootstrap {} must be array", table))?;
        if table == "map_nodes" && rows.is_empty() {
            return Err("bootstrap map_nodes cannot be empty".to_string());
        }
        if table == "story_state" && rows.is_empty() {
            return Err("bootstrap story_state cannot be empty".to_string());
        }
        for row in rows {
            let Some(map) = row.as_object() else {
                return Err(format!("bootstrap {} row must be object", table));
            };
            validate_patch_row(table, map)?;
        }
    }

    Ok(())
}

fn clip(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect()
}
