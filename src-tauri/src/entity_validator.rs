use crate::entity_types::{
    CharacterProfile, EntityCandidateRequest, EntityType, MapNodeDef, ResolvedEntity, TechniqueDef,
    ValidationReport, ValidationStatus,
};
use crate::numeric_guard;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn reject(reason: impl Into<String>) -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Rejected,
        reasons: vec![reason.into()],
        normalized_payload: None,
    }
}

fn accepted(payload: Value) -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Accepted,
        reasons: Vec::new(),
        normalized_payload: Some(payload),
    }
}

fn normalized(payload: Value, reason: impl Into<String>) -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Normalized,
        reasons: vec![reason.into()],
        normalized_payload: Some(payload),
    }
}

fn normalize_root_affinity(values: &[String]) -> Result<(Vec<String>, bool), String> {
    if values.is_empty() {
        return Ok((vec!["Neutral".to_string()], true));
    }

    let allowed = ["Metal", "Wood", "Water", "Fire", "Earth", "Neutral"];
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    let mut changed = false;

    for raw in values {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("rootAffinity contains blank entry".to_string());
        }
        if !allowed.contains(&trimmed) {
            return Err(format!("rootAffinity contains unknown element: {}", trimmed));
        }
        let value = trimmed.to_string();
        if seen.insert(value.clone()) {
            normalized.push(value);
        } else {
            changed = true;
        }
    }

    Ok((normalized, changed))
}

fn normalize_risk_tags(values: &[String]) -> Result<(Vec<String>, bool), String> {
    if values.is_empty() {
        return Ok((Vec::new(), false));
    }

    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    let mut changed = false;

    for raw in values {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("riskTags contains blank entry".to_string());
        }
        let lower = trimmed.to_ascii_lowercase();
        if seen.insert(lower.clone()) {
            normalized.push(lower);
        } else {
            changed = true;
        }
    }

    Ok((normalized, changed))
}

pub fn resolve_candidate(candidate: &EntityCandidateRequest) -> ResolvedEntity {
    match candidate.entity_type {
        EntityType::Technique => resolve_technique(candidate),
        EntityType::Character => resolve_character(candidate),
        EntityType::MapNode => resolve_map_node(candidate),
        EntityType::Item => resolve_item(candidate),
    }
}

fn resolve_technique(candidate: &EntityCandidateRequest) -> ResolvedEntity {
    let parsed: Result<TechniqueDef, _> = serde_json::from_value(candidate.payload.clone());
    if let Err(err) = parsed {
        return ResolvedEntity {
            entity_id: "invalid-technique".to_string(),
            entity_type: EntityType::Technique,
            payload: candidate.payload.clone(),
            validation_report: reject(format!("invalid technique payload: {}", err)),
        };
    }
    let mut t = parsed.unwrap();
    if blank(&t.technique_id) || blank(&t.name) || blank(&t.description) {
        return ResolvedEntity {
            entity_id: if t.technique_id.is_empty() {
                "invalid-technique".to_string()
            } else {
                t.technique_id.clone()
            },
            entity_type: EntityType::Technique,
            payload: json!(t),
            validation_report: reject("technique_id/name/description must be non-empty"),
        };
    }

    let (root_affinity, root_changed) = match normalize_root_affinity(&t.root_affinity) {
        Ok(v) => v,
        Err(reason) => {
            return ResolvedEntity {
                entity_id: t.technique_id.clone(),
                entity_type: EntityType::Technique,
                payload: json!(t),
                validation_report: reject(reason),
            };
        }
    };
    t.root_affinity = root_affinity;
    let (risk_tags, risk_changed) = match normalize_risk_tags(&t.risk_tags) {
        Ok(v) => v,
        Err(reason) => {
            return ResolvedEntity {
                entity_id: t.technique_id.clone(),
                entity_type: EntityType::Technique,
                payload: json!(t),
                validation_report: reject(reason),
            };
        }
    };
    t.risk_tags = risk_tags;

    let check = numeric_guard::validate_technique_power(t.realm_requirement, t.base_power);
    let mut report = if !check.accepted {
        reject(check.reason.unwrap_or_else(|| "technique numeric validation failed".to_string()))
    } else if check.normalized {
        if let Some(value) = check.normalized_value {
            t.base_power = value;
        }
        normalized(
            json!(t),
            check.reason.unwrap_or_else(|| "technique numeric normalized".to_string()),
        )
    } else {
        accepted(json!(t))
    };

    if t.tags.is_empty() {
        t.tags.push("general".to_string());
        report.status = ValidationStatus::Normalized;
        report
            .reasons
            .push("empty tags normalized to ['general']".to_string());
        report.normalized_payload = Some(json!(t));
    }

    if root_changed {
        report.status = ValidationStatus::Normalized;
        report
            .reasons
            .push("duplicate rootAffinity entries deduplicated".to_string());
        report.normalized_payload = Some(json!(t));
    }
    if risk_changed {
        report.status = ValidationStatus::Normalized;
        report
            .reasons
            .push("riskTags normalized to lower-case and deduplicated".to_string());
        report.normalized_payload = Some(json!(t));
    }

    ResolvedEntity {
        entity_id: t.technique_id,
        entity_type: EntityType::Technique,
        payload: report
            .normalized_payload
            .clone()
            .unwrap_or_else(|| candidate.payload.clone()),
        validation_report: report,
    }
}

fn resolve_character(candidate: &EntityCandidateRequest) -> ResolvedEntity {
    let parsed: Result<CharacterProfile, _> = serde_json::from_value(candidate.payload.clone());
    if let Err(err) = parsed {
        return ResolvedEntity {
            entity_id: "invalid-character".to_string(),
            entity_type: EntityType::Character,
            payload: candidate.payload.clone(),
            validation_report: reject(format!("invalid character payload: {}", err)),
        };
    }
    let c = parsed.unwrap();
    let report = if blank(&c.character_id) || blank(&c.name) || blank(&c.realm) {
        reject("character_id/name/realm must be non-empty")
    } else {
        accepted(json!(c))
    };

    ResolvedEntity {
        entity_id: c.character_id,
        entity_type: EntityType::Character,
        payload: report
            .normalized_payload
            .clone()
            .unwrap_or_else(|| candidate.payload.clone()),
        validation_report: report,
    }
}

fn resolve_map_node(candidate: &EntityCandidateRequest) -> ResolvedEntity {
    let parsed: Result<MapNodeDef, _> = serde_json::from_value(candidate.payload.clone());
    if let Err(err) = parsed {
        return ResolvedEntity {
            entity_id: "invalid-map-node".to_string(),
            entity_type: EntityType::MapNode,
            payload: candidate.payload.clone(),
            validation_report: reject(format!("invalid map node payload: {}", err)),
        };
    }

    let mut n = parsed.unwrap();
    if blank(&n.node_id) || blank(&n.name) || blank(&n.node_type) {
        return ResolvedEntity {
            entity_id: if n.node_id.is_empty() {
                "invalid-map-node".to_string()
            } else {
                n.node_id.clone()
            },
            entity_type: EntityType::MapNode,
            payload: json!(n),
            validation_report: reject("node_id/name/node_type must be non-empty"),
        };
    }

    let check = numeric_guard::validate_map_numbers(n.danger_tier, n.aura_density);
    let report = if !check.accepted {
        reject(check.reason.unwrap_or_else(|| "map numeric validation failed".to_string()))
    } else if check.normalized {
        if let Some(v) = check.normalized_value {
            n.aura_density = v;
        }
        normalized(
            json!(n),
            check.reason.unwrap_or_else(|| "map numeric normalized".to_string()),
        )
    } else {
        accepted(json!(n))
    };

    ResolvedEntity {
        entity_id: n.node_id,
        entity_type: EntityType::MapNode,
        payload: report
            .normalized_payload
            .clone()
            .unwrap_or_else(|| candidate.payload.clone()),
        validation_report: report,
    }
}

fn resolve_item(candidate: &EntityCandidateRequest) -> ResolvedEntity {
    let payload = candidate.payload.clone();
    let id = payload
        .get("itemId")
        .and_then(|v| v.as_str())
        .unwrap_or("invalid-item")
        .to_string();
    let valid = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(|v| !blank(v))
        .unwrap_or(false);
    let report = if valid {
        accepted(payload.clone())
    } else {
        reject("item payload requires non-empty name")
    };

    ResolvedEntity {
        entity_id: id,
        entity_type: EntityType::Item,
        payload,
        validation_report: report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_types::EntityCandidateRequest;
    use proptest::prelude::*;

    #[test]
    fn normalizes_technique_power_out_of_band() {
        let req = EntityCandidateRequest {
            entity_type: EntityType::Technique,
            payload: json!({
                "techniqueId": "t_fire_1",
                "name": "Fire Burst",
                "tags": ["fire"],
                "realmRequirement": 1,
                "rootAffinity": ["Fire"],
                "basePower": 999.0,
                "riskTags": [],
                "description": "A basic attack"
            }),
            source_trace_id: None,
        };
        let resolved = resolve_candidate(&req);
        assert!(matches!(
            resolved.validation_report.status,
            ValidationStatus::Normalized
        ));
    }

    #[test]
    fn normalizes_empty_root_affinity_to_neutral() {
        let req = EntityCandidateRequest {
            entity_type: EntityType::Technique,
            payload: json!({
                "techniqueId": "t_root_empty",
                "name": "Rootless Art",
                "tags": ["utility"],
                "realmRequirement": 1,
                "rootAffinity": [],
                "basePower": 20.0,
                "riskTags": [],
                "description": "neutralized affinity"
            }),
            source_trace_id: None,
        };
        let resolved = resolve_candidate(&req);
        assert!(matches!(
            resolved.validation_report.status,
            ValidationStatus::Normalized
        ));
        let arr = resolved
            .payload
            .get("rootAffinity")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], Value::String("Neutral".to_string()));
    }

    #[test]
    fn rejects_unknown_root_affinity_element() {
        let req = EntityCandidateRequest {
            entity_type: EntityType::Technique,
            payload: json!({
                "techniqueId": "t_root_bad",
                "name": "Bad Root Art",
                "tags": ["utility"],
                "realmRequirement": 1,
                "rootAffinity": ["Chaos"],
                "basePower": 20.0,
                "riskTags": [],
                "description": "invalid affinity"
            }),
            source_trace_id: None,
        };
        let resolved = resolve_candidate(&req);
        assert!(matches!(
            resolved.validation_report.status,
            ValidationStatus::Rejected
        ));
    }

    #[test]
    fn normalizes_and_deduplicates_risk_tags() {
        let req = EntityCandidateRequest {
            entity_type: EntityType::Technique,
            payload: json!({
                "techniqueId": "t_risk_norm",
                "name": "Risky Art",
                "tags": ["utility"],
                "realmRequirement": 1,
                "rootAffinity": ["Fire"],
                "basePower": 20.0,
                "riskTags": ["Backlash", "backlash", "Overheat"],
                "description": "risk normalized"
            }),
            source_trace_id: None,
        };
        let resolved = resolve_candidate(&req);
        assert!(matches!(
            resolved.validation_report.status,
            ValidationStatus::Normalized
        ));
        let arr = resolved
            .payload
            .get("riskTags")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let tags = arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        assert_eq!(tags, vec!["backlash", "overheat"]);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_technique_root_affinity_never_contains_blank_or_duplicate(
            affinities in proptest::collection::vec(
                prop_oneof![
                    Just("Metal".to_string()),
                    Just("Wood".to_string()),
                    Just("Water".to_string()),
                    Just("Fire".to_string()),
                    Just("Earth".to_string()),
                    Just("Neutral".to_string()),
                    Just("".to_string())
                ],
                0..8
            )
        ) {
            let req = EntityCandidateRequest {
                entity_type: EntityType::Technique,
                payload: json!({
                    "techniqueId": "t_prop_root",
                    "name": "Prop Root",
                    "tags": ["utility"],
                    "realmRequirement": 1,
                    "rootAffinity": affinities,
                    "basePower": 20.0,
                    "riskTags": [],
                    "description": "property"
                }),
                source_trace_id: None,
            };
            let resolved = resolve_candidate(&req);
            if matches!(resolved.validation_report.status, ValidationStatus::Rejected) {
                let reason = resolved.validation_report.reasons.join(" ");
                prop_assert!(reason.contains("rootAffinity"));
            } else {
                let arr = resolved
                    .payload
                    .get("rootAffinity")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let strings = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>();
                prop_assert!(!strings.is_empty());
                prop_assert!(strings.iter().all(|s| !s.trim().is_empty()));
                let uniq = strings.iter().copied().collect::<std::collections::BTreeSet<_>>();
                prop_assert_eq!(uniq.len(), strings.len());
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_technique_risk_tags_are_lowercase_and_unique(
            risk_tags in proptest::collection::vec(
                prop_oneof![
                    Just("Backlash".to_string()),
                    Just("backlash".to_string()),
                    Just("Overheat".to_string()),
                    Just("Deviation".to_string()),
                    Just("".to_string())
                ],
                0..8
            )
        ) {
            let req = EntityCandidateRequest {
                entity_type: EntityType::Technique,
                payload: json!({
                    "techniqueId": "t_prop_risk",
                    "name": "Prop Risk",
                    "tags": ["utility"],
                    "realmRequirement": 1,
                    "rootAffinity": ["Fire"],
                    "basePower": 20.0,
                    "riskTags": risk_tags,
                    "description": "property"
                }),
                source_trace_id: None,
            };
            let resolved = resolve_candidate(&req);
            if matches!(resolved.validation_report.status, ValidationStatus::Rejected) {
                let reason = resolved.validation_report.reasons.join(" ");
                prop_assert!(reason.contains("riskTags"));
            } else {
                let arr = resolved
                    .payload
                    .get("riskTags")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let tags = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>();
                let uniq = tags.iter().copied().collect::<std::collections::BTreeSet<_>>();
                prop_assert_eq!(uniq.len(), tags.len());
                prop_assert!(tags.iter().all(|t| t == &t.to_ascii_lowercase()));
            }
        }
    }
}
