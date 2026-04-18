use crate::noname_knowledge_store::{
    GraphKnowledgeProvider, InMemoryKnowledgeProvider, NoNameKnowledgeDocument,
    NoNameKnowledgeGraphEdge, NoNameKnowledgeGraphNode, NoNameKnowledgeProvider,
    NoNameKnowledgeQuery, NoNameKnowledgeSnippet,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeProviderReport {
    pub provider_id: String,
    pub returned_count: usize,
    pub accepted_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeRetrievalReport {
    pub provider_count: usize,
    pub total_returned_count: usize,
    pub total_accepted_before_limit: usize,
    pub duplicate_dropped_count: usize,
    pub dropped_by_limit_count: usize,
    pub limit: usize,
    pub provider_reports: Vec<NoNameKnowledgeProviderReport>,
    pub snippets: Vec<NoNameKnowledgeSnippet>,
}

#[derive(Default)]
pub struct NoNameKnowledgeRetrievalService {
    providers: Vec<Box<dyn NoNameKnowledgeProvider>>,
}

impl NoNameKnowledgeRetrievalService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&mut self, provider: Box<dyn NoNameKnowledgeProvider>) {
        self.providers.push(provider);
    }

    pub fn with_in_memory_documents(documents: Vec<NoNameKnowledgeDocument>) -> Self {
        let mut service = Self::new();
        service.register_provider(Box::new(InMemoryKnowledgeProvider::new(documents)));
        service
    }

    pub fn with_graph_knowledge(
        nodes: Vec<NoNameKnowledgeGraphNode>,
        edges: Vec<NoNameKnowledgeGraphEdge>,
    ) -> Self {
        let mut service = Self::new();
        service.register_provider(Box::new(GraphKnowledgeProvider::new(nodes, edges)));
        service
    }

    pub fn retrieve(&self, query: &NoNameKnowledgeQuery) -> Vec<NoNameKnowledgeSnippet> {
        self.retrieve_with_report(query).snippets
    }

    pub fn retrieve_with_report(
        &self,
        query: &NoNameKnowledgeQuery,
    ) -> NoNameKnowledgeRetrievalReport {
        let provider_limit = expanded_provider_limit(query.limit, self.providers.len());
        let mut provider_query = query.clone();
        provider_query.limit = provider_limit;

        let mut provider_reports = Vec::new();
        let mut total_returned_count = 0;
        let mut snippets = Vec::new();

        for provider in &self.providers {
            let provider_id = provider.provider_id().to_string();
            let returned = provider.retrieve(&provider_query);
            let returned_count = returned.len();
            total_returned_count += returned_count;

            let accepted = returned
                .into_iter()
                .filter(|snippet| source_matches(snippet, &query.sources))
                .filter(|snippet| min_score_matches(snippet, query.min_score))
                .collect::<Vec<_>>();
            let accepted_count = accepted.len();
            snippets.extend(accepted);
            provider_reports.push(NoNameKnowledgeProviderReport {
                provider_id,
                returned_count,
                accepted_count,
            });
        }

        let total_accepted_before_limit = snippets.len();
        let (mut snippets, duplicate_dropped_count) = dedupe_snippets(snippets);
        snippets.sort_by(compare_snippets);
        let dropped_by_limit_count = snippets.len().saturating_sub(query.limit);
        snippets.truncate(query.limit);

        NoNameKnowledgeRetrievalReport {
            provider_count: self.providers.len(),
            total_returned_count,
            total_accepted_before_limit,
            duplicate_dropped_count,
            dropped_by_limit_count,
            limit: query.limit,
            provider_reports,
            snippets,
        }
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

fn expanded_provider_limit(limit: usize, provider_count: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    limit.saturating_mul(provider_count.max(1))
}

fn source_matches(snippet: &NoNameKnowledgeSnippet, sources: &[String]) -> bool {
    sources.is_empty()
        || sources
            .iter()
            .any(|source| snippet.source.eq_ignore_ascii_case(source))
}

fn min_score_matches(snippet: &NoNameKnowledgeSnippet, min_score: Option<u32>) -> bool {
    min_score
        .map(|threshold| snippet.score >= threshold)
        .unwrap_or(true)
}

fn dedupe_snippets(snippets: Vec<NoNameKnowledgeSnippet>) -> (Vec<NoNameKnowledgeSnippet>, usize) {
    let mut deduped: Vec<NoNameKnowledgeSnippet> = Vec::new();
    let mut duplicate_count = 0;

    for snippet in snippets {
        let key = snippet_key(&snippet);
        if let Some(existing) = deduped.iter_mut().find(|item| snippet_key(item) == key) {
            duplicate_count += 1;
            if compare_snippets(&snippet, existing).is_lt() {
                *existing = snippet;
            }
        } else {
            deduped.push(snippet);
        }
    }

    (deduped, duplicate_count)
}

fn snippet_key(snippet: &NoNameKnowledgeSnippet) -> String {
    format!(
        "{}:{}",
        snippet.source.to_lowercase(),
        snippet.document_id.to_lowercase()
    )
}

fn compare_snippets(
    left: &NoNameKnowledgeSnippet,
    right: &NoNameKnowledgeSnippet,
) -> std::cmp::Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.provider_id.cmp(&right.provider_id))
        .then_with(|| left.document_id.cmp(&right.document_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieval_service_can_query_registered_provider() {
        let service = NoNameKnowledgeRetrievalService::with_in_memory_documents(vec![
            NoNameKnowledgeDocument {
                document_id: "doc-1".to_string(),
                title: "青云山门志".to_string(),
                body: "山门法阵可抵御大多数外敌。".to_string(),
                tags: vec!["山门".to_string(), "法阵".to_string()],
                source: "lore".to_string(),
            },
            NoNameKnowledgeDocument {
                document_id: "doc-2".to_string(),
                title: "灵田管理录".to_string(),
                body: "灵田由杂役弟子日常照料。".to_string(),
                tags: vec!["灵田".to_string()],
                source: "lore".to_string(),
            },
        ]);

        let snippets = service.retrieve(&NoNameKnowledgeQuery {
            keyword: Some("法阵".to_string()),
            tags: vec!["山门".to_string()],
            sources: vec![],
            min_score: None,
            limit: 3,
        });

        assert_eq!(service.provider_count(), 1);
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].document_id, "doc-1");
    }

    #[test]
    fn retrieval_service_respects_limit() {
        let service = NoNameKnowledgeRetrievalService::with_in_memory_documents(vec![
            NoNameKnowledgeDocument {
                document_id: "doc-1".to_string(),
                title: "山门志".to_string(),
                body: "山门法阵。".to_string(),
                tags: vec!["山门".to_string()],
                source: "lore".to_string(),
            },
            NoNameKnowledgeDocument {
                document_id: "doc-2".to_string(),
                title: "山门夜巡".to_string(),
                body: "夜巡弟子巡逻。".to_string(),
                tags: vec!["山门".to_string()],
                source: "lore".to_string(),
            },
        ]);

        let snippets = service.retrieve(&NoNameKnowledgeQuery {
            keyword: Some("山门".to_string()),
            tags: vec![],
            sources: vec![],
            min_score: None,
            limit: 1,
        });

        assert_eq!(snippets.len(), 1);
    }

    #[test]
    fn retrieval_service_can_query_graph_provider() {
        let service = NoNameKnowledgeRetrievalService::with_graph_knowledge(
            vec![
                NoNameKnowledgeGraphNode {
                    node_id: "sect".to_string(),
                    label: "Qingyun Sect".to_string(),
                    body: "A sect with a sealed forbidden valley.".to_string(),
                    tags: vec!["sect".to_string()],
                    source: "graph-demo".to_string(),
                },
                NoNameKnowledgeGraphNode {
                    node_id: "valley".to_string(),
                    label: "Forbidden Valley".to_string(),
                    body: "A dangerous valley below the main peak.".to_string(),
                    tags: vec!["location".to_string()],
                    source: "graph-demo".to_string(),
                },
            ],
            vec![NoNameKnowledgeGraphEdge {
                from: "sect".to_string(),
                to: "valley".to_string(),
                relation: "seals".to_string(),
                weight: 4,
            }],
        );

        let snippets = service.retrieve(&NoNameKnowledgeQuery {
            keyword: Some("seals".to_string()),
            tags: vec![],
            sources: vec![],
            min_score: None,
            limit: 4,
        });

        assert_eq!(service.provider_count(), 1);
        assert_eq!(snippets.len(), 2);
        assert!(snippets.iter().all(|item| item.provider_id == "graph_lore"));
    }

    #[test]
    fn retrieval_report_tracks_provider_counts_and_source_filtering() {
        let service = NoNameKnowledgeRetrievalService::with_in_memory_documents(vec![
            NoNameKnowledgeDocument {
                document_id: "doc-lore".to_string(),
                title: "Qingyun Gate Array".to_string(),
                body: "The gate array protects the main path.".to_string(),
                tags: vec!["array".to_string()],
                source: "lore".to_string(),
            },
            NoNameKnowledgeDocument {
                document_id: "doc-rule".to_string(),
                title: "Qingyun Gate Patrol Rule".to_string(),
                body: "Outer disciples patrol the gate.".to_string(),
                tags: vec!["rule".to_string()],
                source: "rules".to_string(),
            },
        ]);

        let report = service.retrieve_with_report(&NoNameKnowledgeQuery {
            keyword: Some("gate".to_string()),
            tags: vec![],
            sources: vec!["rules".to_string()],
            min_score: None,
            limit: 5,
        });

        assert_eq!(report.provider_count, 1);
        assert_eq!(report.total_returned_count, 2);
        assert_eq!(report.total_accepted_before_limit, 1);
        assert_eq!(report.provider_reports[0].returned_count, 2);
        assert_eq!(report.provider_reports[0].accepted_count, 1);
        assert_eq!(report.snippets.len(), 1);
        assert_eq!(report.snippets[0].document_id, "doc-rule");
    }

    #[test]
    fn retrieval_report_filters_weak_snippets_by_min_score() {
        let service = NoNameKnowledgeRetrievalService::with_in_memory_documents(vec![
            NoNameKnowledgeDocument {
                document_id: "doc-strong".to_string(),
                title: "Gate Array".to_string(),
                body: "Gate array keeps the sect path sealed.".to_string(),
                tags: vec!["gate".to_string()],
                source: "lore".to_string(),
            },
            NoNameKnowledgeDocument {
                document_id: "doc-weak".to_string(),
                title: "Gate Rumor".to_string(),
                body: "A short rumor mentions the gate.".to_string(),
                tags: vec![],
                source: "lore".to_string(),
            },
        ]);

        let report = service.retrieve_with_report(&NoNameKnowledgeQuery {
            keyword: Some("gate".to_string()),
            tags: vec!["gate".to_string()],
            sources: vec![],
            min_score: Some(9),
            limit: 5,
        });

        assert_eq!(report.total_returned_count, 2);
        assert_eq!(report.total_accepted_before_limit, 1);
        assert_eq!(report.provider_reports[0].accepted_count, 1);
        assert_eq!(report.snippets.len(), 1);
        assert_eq!(report.snippets[0].document_id, "doc-strong");
    }

    #[test]
    fn retrieval_report_dedupes_same_source_and_document_id() {
        let mut service = NoNameKnowledgeRetrievalService::new();
        service.register_provider(Box::new(InMemoryKnowledgeProvider::new(vec![
            NoNameKnowledgeDocument {
                document_id: "shared-doc".to_string(),
                title: "Gate Note".to_string(),
                body: "A brief note mentions the gate.".to_string(),
                tags: vec![],
                source: "lore".to_string(),
            },
        ])));
        service.register_provider(Box::new(InMemoryKnowledgeProvider::new(vec![
            NoNameKnowledgeDocument {
                document_id: "shared-doc".to_string(),
                title: "Gate Array Anchor".to_string(),
                body: "Gate array keeps the sect path sealed.".to_string(),
                tags: vec!["gate".to_string()],
                source: "lore".to_string(),
            },
        ])));

        let report = service.retrieve_with_report(&NoNameKnowledgeQuery {
            keyword: Some("gate".to_string()),
            tags: vec!["gate".to_string()],
            sources: vec![],
            min_score: None,
            limit: 5,
        });

        assert_eq!(report.provider_count, 2);
        assert_eq!(report.total_returned_count, 2);
        assert_eq!(report.total_accepted_before_limit, 2);
        assert_eq!(report.duplicate_dropped_count, 1);
        assert_eq!(report.dropped_by_limit_count, 0);
        assert_eq!(report.snippets.len(), 1);
        assert_eq!(report.snippets[0].title, "Gate Array Anchor");
    }
}
