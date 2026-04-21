use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameMemoryImportance, NoNameNarrativeMemoryItem,
    NoNameNarrativeNoteType, NoNameNarrativeStatus, NoNameSemanticMemoryItem,
    NoNameWorkingMemoryItem,
};
use crate::noname_types::NoNameRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameMemoryQuery {
    pub role: NoNameRole,
    pub search_term: Option<String>,
    pub actor: Option<String>,
    pub location: Option<String>,
    pub goal: Option<String>,
    pub keyword: Option<String>,
    pub token_budget: usize,
    pub per_section_limit: usize,
}

impl NoNameMemoryQuery {
    pub fn by_keyword(
        role: NoNameRole,
        keyword: impl Into<String>,
        token_budget: usize,
        per_section_limit: usize,
    ) -> Self {
        Self {
            role,
            search_term: None,
            actor: None,
            location: None,
            goal: None,
            keyword: Some(keyword.into()),
            token_budget,
            per_section_limit,
        }
    }

    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = Some(goal.into());
        self
    }

    pub fn has_structured_filters(&self) -> bool {
        self.actor.is_some()
            || self.location.is_some()
            || self.goal.is_some()
            || self.keyword.is_some()
            || self.search_term.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoNameRetrievedMemories {
    pub working: Vec<NoNameWorkingMemoryItem>,
    pub episodic: Vec<NoNameEpisodicMemoryItem>,
    pub semantic: Vec<NoNameSemanticMemoryItem>,
    pub narrative: Vec<NoNameNarrativeMemoryItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoNameMemorySection {
    Working,
    Episodic,
    Semantic,
    Narrative,
}

impl NoNameMemorySection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Narrative => "narrative",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NoNameRetrievalScore {
    pub relevance: u32,
    pub recency: u64,
    pub importance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameRetrievalExplanation {
    pub section: NoNameMemorySection,
    pub item_id: String,
    pub rank: usize,
    pub raw_match: u32,
    pub role_boost: u32,
    pub score: NoNameRetrievalScore,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoNameRetrievedMemoryReport {
    pub memories: NoNameRetrievedMemories,
    pub explanations: Vec<NoNameRetrievalExplanation>,
}

pub fn retrieve_memories(
    query: &NoNameMemoryQuery,
    working: &[NoNameWorkingMemoryItem],
    episodic: &[NoNameEpisodicMemoryItem],
    semantic: &[NoNameSemanticMemoryItem],
    narrative: &[NoNameNarrativeMemoryItem],
) -> NoNameRetrievedMemories {
    retrieve_memories_with_explanations(query, working, episodic, semantic, narrative).memories
}

pub fn retrieve_memories_with_explanations(
    query: &NoNameMemoryQuery,
    working: &[NoNameWorkingMemoryItem],
    episodic: &[NoNameEpisodicMemoryItem],
    semantic: &[NoNameSemanticMemoryItem],
    narrative: &[NoNameNarrativeMemoryItem],
) -> NoNameRetrievedMemoryReport {
    let working_items = rank_working(query, working);
    let episodic_items = rank_episodic(query, episodic);
    let semantic_items = rank_semantic(query, semantic);
    let narrative_items = rank_narrative(query, narrative);

    let explanations = working_items
        .explanations
        .iter()
        .chain(episodic_items.explanations.iter())
        .chain(semantic_items.explanations.iter())
        .chain(narrative_items.explanations.iter())
        .cloned()
        .collect();

    NoNameRetrievedMemoryReport {
        memories: NoNameRetrievedMemories {
            working: working_items.items,
            episodic: episodic_items.items,
            semantic: semantic_items.items,
            narrative: narrative_items.items,
        },
        explanations,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RankedSection<T> {
    items: Vec<T>,
    explanations: Vec<NoNameRetrievalExplanation>,
}

fn rank_working(
    query: &NoNameMemoryQuery,
    working: &[NoNameWorkingMemoryItem],
) -> RankedSection<NoNameWorkingMemoryItem> {
    rank_items(
        NoNameMemorySection::Working,
        working.iter().enumerate(),
        query,
        |(_, item)| item.memory_id.clone(),
        |(_, item)| working_match_score(query, item),
        |(index, item), raw_match| NoNameRetrievalScore {
            relevance: raw_match + role_section_boost(query.role, "working"),
            recency: (*index + 1) as u64,
            importance: item.priority as u32,
        },
        |(_, item)| (*item).clone(),
    )
}

fn rank_episodic(
    query: &NoNameMemoryQuery,
    episodic: &[NoNameEpisodicMemoryItem],
) -> RankedSection<NoNameEpisodicMemoryItem> {
    rank_items(
        NoNameMemorySection::Episodic,
        episodic.iter(),
        query,
        |item| item.memory_id.clone(),
        |item| episodic_match_score(query, item),
        |item, raw_match| NoNameRetrievalScore {
            relevance: raw_match + role_section_boost(query.role, "episodic"),
            recency: item.timestamp,
            importance: importance_weight(item.importance),
        },
        |item| (*item).clone(),
    )
}

fn rank_semantic(
    query: &NoNameMemoryQuery,
    semantic: &[NoNameSemanticMemoryItem],
) -> RankedSection<NoNameSemanticMemoryItem> {
    rank_items(
        NoNameMemorySection::Semantic,
        semantic.iter(),
        query,
        |item| item.fact_id.clone(),
        |item| semantic_match_score(query, item),
        |item, raw_match| NoNameRetrievalScore {
            relevance: raw_match + role_section_boost(query.role, "semantic"),
            recency: item.updated_at,
            importance: item.confidence as u32,
        },
        |item| (*item).clone(),
    )
}

fn rank_narrative(
    query: &NoNameMemoryQuery,
    narrative: &[NoNameNarrativeMemoryItem],
) -> RankedSection<NoNameNarrativeMemoryItem> {
    rank_items(
        NoNameMemorySection::Narrative,
        narrative
            .iter()
            .filter(|item| item.status == NoNameNarrativeStatus::Active),
        query,
        |item| item.note_id.clone(),
        |item| narrative_match_score(query, item),
        |item, raw_match| NoNameRetrievalScore {
            relevance: raw_match + role_section_boost(query.role, "narrative"),
            recency: item.updated_at,
            importance: narrative_importance_weight(item.note_type),
        },
        |item| (*item).clone(),
    )
}

fn rank_items<I, T, U, FId, FMatch, FScore, FClone>(
    section: NoNameMemorySection,
    items: I,
    query: &NoNameMemoryQuery,
    item_id: FId,
    match_score: FMatch,
    score: FScore,
    clone_item: FClone,
) -> RankedSection<U>
where
    I: IntoIterator<Item = T>,
    FId: Fn(&T) -> String,
    FMatch: Fn(&T) -> u32,
    FScore: Fn(&T, u32) -> NoNameRetrievalScore,
    FClone: Fn(&T) -> U,
{
    let mut ranked = items
        .into_iter()
        .filter_map(|item| {
            let raw_match = match_score(&item);
            if query.has_structured_filters() && raw_match == 0 {
                return None;
            }
            let score = score(&item, raw_match);
            let role_boost = score.relevance.saturating_sub(raw_match);
            let explanation = build_retrieval_explanation(
                section,
                item_id(&item),
                raw_match,
                role_boost,
                score,
                query,
            );
            Some((score, clone_item(&item), explanation))
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|item| std::cmp::Reverse(item.0));
    ranked.truncate(query.per_section_limit);

    let mut items = Vec::with_capacity(ranked.len());
    let mut explanations = Vec::with_capacity(ranked.len());
    for (index, (_, item, mut explanation)) in ranked.into_iter().enumerate() {
        explanation.rank = index + 1;
        items.push(item);
        explanations.push(explanation);
    }

    RankedSection {
        items,
        explanations,
    }
}

fn build_retrieval_explanation(
    section: NoNameMemorySection,
    item_id: String,
    raw_match: u32,
    role_boost: u32,
    score: NoNameRetrievalScore,
    query: &NoNameMemoryQuery,
) -> NoNameRetrievalExplanation {
    let mut reasons = vec![
        "sort=relevance_desc,recency_desc,importance_desc".to_string(),
        format!("section={}", section.as_str()),
        format!("relevance={raw_match}+{role_boost}"),
        format!("recency={}", score.recency),
        format!("importance={}", score.importance),
    ];

    if query.has_structured_filters() {
        reasons.push(format!("matched_filters_score={raw_match}"));
    } else {
        reasons.push("default_query_keeps_recent_memory_visible".to_string());
    }
    if role_boost > 0 {
        reasons.push(format!("role_boost={role_boost}"));
    }

    NoNameRetrievalExplanation {
        section,
        item_id,
        rank: 0,
        raw_match,
        role_boost,
        score,
        reasons,
    }
}

fn working_match_score(query: &NoNameMemoryQuery, item: &NoNameWorkingMemoryItem) -> u32 {
    let content = [
        item.summary.as_str(),
        item.category.as_str(),
        item.source.as_str(),
    ];
    score_text_fields(query, &content)
}

fn episodic_match_score(query: &NoNameMemoryQuery, item: &NoNameEpisodicMemoryItem) -> u32 {
    let actor_score = query
        .actor
        .as_deref()
        .map(|actor| contains_any(&item.actors, actor) as u32 * 5)
        .unwrap_or_default();
    let location_score = query
        .location
        .as_deref()
        .map(|location| {
            item.location_id
                .as_deref()
                .map(|value| contains_text(value, location))
                .unwrap_or(false) as u32
                * 5
        })
        .unwrap_or_default();
    let text_score = score_text_fields(
        query,
        &[
            item.summary.as_str(),
            item.event_type.as_str(),
            item.detail_ref.as_deref().unwrap_or(""),
        ],
    );

    actor_score + location_score + text_score
}

fn semantic_match_score(query: &NoNameMemoryQuery, item: &NoNameSemanticMemoryItem) -> u32 {
    let text_score = score_text_fields(
        query,
        &[
            item.subject.as_str(),
            item.predicate.as_str(),
            item.object.as_str(),
            &item.tags.join(" "),
        ],
    );
    let actor_score = query
        .actor
        .as_deref()
        .map(|actor| {
            (contains_text(&item.subject, actor) || contains_text(&item.object, actor)) as u32 * 5
        })
        .unwrap_or_default();
    let location_score = query
        .location
        .as_deref()
        .map(|location| {
            (contains_text(&item.subject, location)
                || contains_text(&item.object, location)
                || contains_any(&item.tags, location)) as u32
                * 5
        })
        .unwrap_or_default();

    actor_score + location_score + text_score
}

fn narrative_match_score(query: &NoNameMemoryQuery, item: &NoNameNarrativeMemoryItem) -> u32 {
    let actor_score = query
        .actor
        .as_deref()
        .map(|actor| contains_any(&item.related_entities, actor) as u32 * 5)
        .unwrap_or_default();
    let text_score = score_text_fields(query, &[item.title.as_str(), item.summary.as_str()]);

    actor_score + text_score
}

fn score_text_fields(query: &NoNameMemoryQuery, fields: &[&str]) -> u32 {
    let mut score = 0;

    if let Some(term) = query.keyword.as_deref().or(query.search_term.as_deref()) {
        score += contains_in_fields(fields, term) as u32 * 3;
    }
    if let Some(goal) = query.goal.as_deref() {
        score += contains_in_fields(fields, goal) as u32 * 4;
    }
    if let Some(actor) = query.actor.as_deref() {
        score += contains_in_fields(fields, actor) as u32 * 2;
    }
    if let Some(location) = query.location.as_deref() {
        score += contains_in_fields(fields, location) as u32 * 2;
    }

    if !query.has_structured_filters() {
        score = 1;
    }

    score
}

fn contains_in_fields(fields: &[&str], needle: &str) -> bool {
    fields.iter().any(|field| contains_text(field, needle))
}

fn contains_any(items: &[String], needle: &str) -> bool {
    items.iter().any(|item| contains_text(item, needle))
}

fn contains_text(content: &str, search_term: &str) -> bool {
    content.to_lowercase().contains(&search_term.to_lowercase())
}

fn importance_weight(importance: NoNameMemoryImportance) -> u32 {
    match importance {
        NoNameMemoryImportance::Low => 20,
        NoNameMemoryImportance::Medium => 50,
        NoNameMemoryImportance::High => 90,
    }
}

fn narrative_importance_weight(note_type: NoNameNarrativeNoteType) -> u32 {
    match note_type {
        NoNameNarrativeNoteType::Goal => 70,
        NoNameNarrativeNoteType::Conflict => 90,
        NoNameNarrativeNoteType::Foreshadowing => 50,
        NoNameNarrativeNoteType::UnresolvedThread => 85,
        NoNameNarrativeNoteType::CharacterArc => 65,
    }
}

fn role_section_boost(role: NoNameRole, section: &str) -> u32 {
    match (role, section) {
        (NoNameRole::Director, "narrative") => 4,
        (NoNameRole::Director, "episodic") => 3,
        (NoNameRole::WorldCurator, "semantic") => 5,
        (NoNameRole::WorldCurator, "narrative") => 2,
        (NoNameRole::NpcIntent, "episodic") => 4,
        (NoNameRole::NpcIntent, "narrative") => 3,
        (NoNameRole::CombatNarrator, "episodic") => 4,
        (NoNameRole::CombatNarrator, "working") => 3,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_memory_types::{NoNameNarrativeStatus, NoNameWorkingMemoryItem};

    fn sample_working(summary: &str, priority: u8) -> NoNameWorkingMemoryItem {
        NoNameWorkingMemoryItem {
            memory_id: format!("work-{priority}"),
            turn_id: format!("turn-{priority}"),
            source: "test".to_string(),
            category: "recent_turn".to_string(),
            summary: summary.to_string(),
            expires_at: None,
            priority,
        }
    }

    fn sample_episodic(
        memory_id: &str,
        actors: Vec<&str>,
        location_id: Option<&str>,
        summary: &str,
        timestamp: u64,
        importance: NoNameMemoryImportance,
    ) -> NoNameEpisodicMemoryItem {
        NoNameEpisodicMemoryItem {
            memory_id: memory_id.to_string(),
            event_type: "battle".to_string(),
            timestamp,
            chapter_index: 1,
            location_id: location_id.map(str::to_string),
            actors: actors.into_iter().map(str::to_string).collect(),
            summary: summary.to_string(),
            detail_ref: None,
            importance,
        }
    }

    fn sample_semantic(
        fact_id: &str,
        subject: &str,
        object: &str,
        confidence: u8,
        updated_at: u64,
        tags: Vec<&str>,
    ) -> NoNameSemanticMemoryItem {
        NoNameSemanticMemoryItem {
            fact_id: fact_id.to_string(),
            subject: subject.to_string(),
            predicate: "位于".to_string(),
            object: object.to_string(),
            confidence,
            source: "test".to_string(),
            updated_at,
            tags: tags.into_iter().map(str::to_string).collect(),
        }
    }

    fn sample_narrative(
        note_id: &str,
        title: &str,
        summary: &str,
        related_entities: Vec<&str>,
        updated_at: u64,
        note_type: NoNameNarrativeNoteType,
    ) -> NoNameNarrativeMemoryItem {
        NoNameNarrativeMemoryItem {
            note_id: note_id.to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type,
            title: title.to_string(),
            summary: summary.to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: related_entities.into_iter().map(str::to_string).collect(),
            updated_at,
        }
    }

    #[test]
    fn retrieval_supports_keyword_actor_location_and_goal_filters() {
        let result = retrieve_memories(
            &NoNameMemoryQuery::by_keyword(NoNameRole::Director, "山门", 200, 3)
                .with_actor("青河长老")
                .with_location("山门")
                .with_goal("守住山门"),
            &[sample_working("青河长老要求死守山门", 8)],
            &[sample_episodic(
                "episode-1",
                vec!["玩家", "青河长老"],
                Some("山门"),
                "青河长老要求弟子守住山门",
                2,
                NoNameMemoryImportance::High,
            )],
            &[sample_semantic(
                "fact-1",
                "青河长老",
                "山门",
                85,
                4,
                vec!["防守"],
            )],
            &[sample_narrative(
                "note-1",
                "守住山门",
                "青河长老要稳住山门防线",
                vec!["青河长老", "山门"],
                5,
                NoNameNarrativeNoteType::Goal,
            )],
        );

        assert_eq!(result.working.len(), 1);
        assert_eq!(result.episodic.len(), 1);
        assert_eq!(result.semantic.len(), 1);
        assert_eq!(result.narrative.len(), 1);
    }

    #[test]
    fn retrieval_ranks_by_relevance_then_recency_then_importance() {
        let result = retrieve_memories(
            &NoNameMemoryQuery::by_keyword(NoNameRole::CombatNarrator, "交锋", 200, 2),
            &[
                sample_working("敌修与玩家激烈交锋", 3),
                sample_working("普通巡山记录", 9),
            ],
            &[
                sample_episodic(
                    "episode-old",
                    vec!["玩家"],
                    Some("山门"),
                    "敌修与玩家交锋，火花四溅",
                    1,
                    NoNameMemoryImportance::High,
                ),
                sample_episodic(
                    "episode-new",
                    vec!["玩家"],
                    Some("山门"),
                    "敌修再次与玩家交锋，剑气更盛",
                    5,
                    NoNameMemoryImportance::Medium,
                ),
            ],
            &[sample_semantic(
                "fact-1",
                "玩家",
                "山门",
                70,
                1,
                vec!["战斗"],
            )],
            &[sample_narrative(
                "note-1",
                "山门交锋",
                "冲突持续升级",
                vec!["玩家"],
                4,
                NoNameNarrativeNoteType::Conflict,
            )],
        );

        assert_eq!(result.episodic.len(), 2);
        assert_eq!(result.episodic[0].memory_id, "episode-new");
        assert_eq!(result.working[0].summary, "敌修与玩家激烈交锋");
    }

    #[test]
    fn retrieval_report_explains_rank_score_components() {
        let report = retrieve_memories_with_explanations(
            &NoNameMemoryQuery::by_keyword(NoNameRole::CombatNarrator, "交锋", 200, 2),
            &[],
            &[
                sample_episodic(
                    "episode-old",
                    vec!["玩家"],
                    Some("山门"),
                    "敌修与玩家交锋，火花四溅",
                    1,
                    NoNameMemoryImportance::High,
                ),
                sample_episodic(
                    "episode-new",
                    vec!["玩家"],
                    Some("山门"),
                    "敌修再次与玩家交锋，剑气更盛",
                    5,
                    NoNameMemoryImportance::Medium,
                ),
            ],
            &[],
            &[],
        );

        assert_eq!(report.memories.episodic[0].memory_id, "episode-new");
        let new_explanation = report
            .explanations
            .iter()
            .find(|item| item.item_id == "episode-new")
            .expect("newer episodic memory should have an explanation");
        assert_eq!(new_explanation.section, NoNameMemorySection::Episodic);
        assert_eq!(new_explanation.rank, 1);
        assert_eq!(new_explanation.raw_match, 3);
        assert_eq!(new_explanation.role_boost, 4);
        assert_eq!(new_explanation.score.relevance, 7);
        assert_eq!(new_explanation.score.recency, 5);
        assert_eq!(new_explanation.score.importance, 50);
        assert!(new_explanation
            .reasons
            .iter()
            .any(|item| item == "sort=relevance_desc,recency_desc,importance_desc"));
    }

    #[test]
    fn retrieval_report_records_role_boost_for_world_curator_semantic_memory() {
        let report = retrieve_memories_with_explanations(
            &NoNameMemoryQuery::by_keyword(NoNameRole::WorldCurator, "山门", 200, 2),
            &[],
            &[],
            &[sample_semantic(
                "fact-1",
                "青河长老",
                "山门",
                88,
                7,
                vec!["地点"],
            )],
            &[],
        );

        assert_eq!(report.memories.semantic[0].fact_id, "fact-1");
        let explanation = report
            .explanations
            .iter()
            .find(|item| item.item_id == "fact-1")
            .expect("semantic memory should have an explanation");
        assert_eq!(explanation.section, NoNameMemorySection::Semantic);
        assert_eq!(explanation.raw_match, 3);
        assert_eq!(explanation.role_boost, 5);
        assert_eq!(explanation.score.relevance, 8);
        assert!(explanation
            .reasons
            .iter()
            .any(|item| item == "role_boost=5"));
    }

    #[test]
    fn retrieval_report_excludes_filtered_out_items_from_explanations() {
        let report = retrieve_memories_with_explanations(
            &NoNameMemoryQuery::by_keyword(NoNameRole::Director, "山门", 200, 2),
            &[
                sample_working("青河长老要求死守山门", 8),
                sample_working("普通巡山记录", 9),
            ],
            &[],
            &[],
            &[],
        );

        assert_eq!(report.memories.working.len(), 1);
        assert_eq!(report.explanations.len(), 1);
        assert_eq!(report.explanations[0].item_id, "work-8");
    }

    #[test]
    fn retrieval_keeps_search_term_backward_compatible() {
        let result = retrieve_memories(
            &NoNameMemoryQuery {
                role: NoNameRole::WorldCurator,
                search_term: Some("山门".to_string()),
                actor: None,
                location: Some("山门".to_string()),
                goal: None,
                keyword: None,
                token_budget: 200,
                per_section_limit: 2,
            },
            &[],
            &[sample_episodic(
                "episode-1",
                vec!["玩家"],
                Some("山门"),
                "玩家返回山门",
                2,
                NoNameMemoryImportance::Medium,
            )],
            &[sample_semantic(
                "fact-1",
                "玩家",
                "山门",
                80,
                3,
                vec!["地点"],
            )],
            &[sample_narrative(
                "note-1",
                "山门危机",
                "强敌逼近",
                vec!["玩家"],
                1,
                NoNameNarrativeNoteType::Conflict,
            )],
        );

        assert_eq!(result.episodic.len(), 1);
        assert_eq!(result.semantic[0].fact_id, "fact-1");
    }
}
