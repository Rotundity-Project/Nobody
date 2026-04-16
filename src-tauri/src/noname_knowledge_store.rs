use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeDocument {
    pub document_id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeQuery {
    pub keyword: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub min_score: Option<u32>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeSnippet {
    pub provider_id: String,
    pub document_id: String,
    pub title: String,
    pub excerpt: String,
    pub score: u32,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeGraphNode {
    pub node_id: String,
    pub label: String,
    pub body: String,
    pub tags: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameKnowledgeGraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: u32,
}

pub trait NoNameKnowledgeProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn retrieve(&self, query: &NoNameKnowledgeQuery) -> Vec<NoNameKnowledgeSnippet>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryKnowledgeProvider {
    documents: Vec<NoNameKnowledgeDocument>,
}

impl InMemoryKnowledgeProvider {
    pub fn new(documents: Vec<NoNameKnowledgeDocument>) -> Self {
        Self { documents }
    }
}

impl NoNameKnowledgeProvider for InMemoryKnowledgeProvider {
    fn provider_id(&self) -> &'static str {
        "in_memory_lore"
    }

    fn retrieve(&self, query: &NoNameKnowledgeQuery) -> Vec<NoNameKnowledgeSnippet> {
        let keyword = query.keyword.as_deref().map(str::to_lowercase);
        let tag_terms = query
            .tags
            .iter()
            .map(|tag| tag.to_lowercase())
            .collect::<Vec<_>>();

        let mut snippets = self
            .documents
            .iter()
            .filter_map(|document| {
                let score = score_document(document, keyword.as_deref(), &tag_terms);
                if score == 0 {
                    return None;
                }

                Some(NoNameKnowledgeSnippet {
                    provider_id: self.provider_id().to_string(),
                    document_id: document.document_id.clone(),
                    title: document.title.clone(),
                    excerpt: build_excerpt(document, keyword.as_deref()),
                    score,
                    source: document.source.clone(),
                })
            })
            .collect::<Vec<_>>();

        snippets.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.document_id.cmp(&b.document_id)));
        snippets.truncate(query.limit);
        snippets
    }
}

#[derive(Debug, Default, Clone)]
pub struct GraphKnowledgeProvider {
    nodes: Vec<NoNameKnowledgeGraphNode>,
    edges: Vec<NoNameKnowledgeGraphEdge>,
}

impl GraphKnowledgeProvider {
    pub fn new(
        nodes: Vec<NoNameKnowledgeGraphNode>,
        edges: Vec<NoNameKnowledgeGraphEdge>,
    ) -> Self {
        Self { nodes, edges }
    }
}

impl NoNameKnowledgeProvider for GraphKnowledgeProvider {
    fn provider_id(&self) -> &'static str {
        "graph_lore"
    }

    fn retrieve(&self, query: &NoNameKnowledgeQuery) -> Vec<NoNameKnowledgeSnippet> {
        let keyword = query.keyword.as_deref().map(str::to_lowercase);
        let tag_terms = query
            .tags
            .iter()
            .map(|tag| tag.to_lowercase())
            .collect::<Vec<_>>();

        let mut snippets = self
            .nodes
            .iter()
            .filter_map(|node| {
                let score = score_graph_node(node, keyword.as_deref(), &tag_terms)
                    + score_graph_edges(node, &self.nodes, &self.edges, keyword.as_deref(), &tag_terms);
                if score == 0 {
                    return None;
                }

                Some(NoNameKnowledgeSnippet {
                    provider_id: self.provider_id().to_string(),
                    document_id: node.node_id.clone(),
                    title: node.label.clone(),
                    excerpt: build_graph_excerpt(node, &self.nodes, &self.edges),
                    score,
                    source: node.source.clone(),
                })
            })
            .collect::<Vec<_>>();

        snippets.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.document_id.cmp(&b.document_id)));
        snippets.truncate(query.limit);
        snippets
    }
}

fn score_document(
    document: &NoNameKnowledgeDocument,
    keyword: Option<&str>,
    tags: &[String],
) -> u32 {
    let mut score = 0;

    if let Some(keyword) = keyword {
        score += contains_text(&document.title, keyword) as u32 * 5;
        score += contains_text(&document.body, keyword) as u32 * 3;
        score += document
            .tags
            .iter()
            .any(|tag| contains_text(tag, keyword)) as u32
            * 2;
    }

    for tag in tags {
        if document.tags.iter().any(|item| contains_text(item, tag)) {
            score += 4;
        }
    }

    if keyword.is_none() && tags.is_empty() {
        score = 1;
    }

    score
}

fn score_graph_node(
    node: &NoNameKnowledgeGraphNode,
    keyword: Option<&str>,
    tags: &[String],
) -> u32 {
    let mut score = 0;

    if let Some(keyword) = keyword {
        score += contains_text(&node.label, keyword) as u32 * 5;
        score += contains_text(&node.body, keyword) as u32 * 3;
        score += node
            .tags
            .iter()
            .any(|tag| contains_text(tag, keyword)) as u32
            * 2;
    }

    for tag in tags {
        if node.tags.iter().any(|item| contains_text(item, tag)) {
            score += 4;
        }
    }

    if keyword.is_none() && tags.is_empty() {
        score = 1;
    }

    score
}

fn score_graph_edges(
    node: &NoNameKnowledgeGraphNode,
    nodes: &[NoNameKnowledgeGraphNode],
    edges: &[NoNameKnowledgeGraphEdge],
    keyword: Option<&str>,
    tags: &[String],
) -> u32 {
    edges
        .iter()
        .filter(|edge| edge.from == node.node_id || edge.to == node.node_id)
        .map(|edge| {
            let neighbor_id = if edge.from == node.node_id { &edge.to } else { &edge.from };
            let neighbor = nodes.iter().find(|item| &item.node_id == neighbor_id);
            let relation_score = keyword
                .map(|term| contains_text(&edge.relation, term) as u32 * 3)
                .unwrap_or_default();
            let neighbor_score = neighbor
                .map(|item| score_graph_node(item, keyword, tags).min(5))
                .unwrap_or_default();
            if relation_score == 0 && neighbor_score == 0 {
                0
            } else {
                relation_score + neighbor_score + edge.weight.min(5)
            }
        })
        .sum()
}

fn build_excerpt(document: &NoNameKnowledgeDocument, keyword: Option<&str>) -> String {
    let text = if let Some(keyword) = keyword {
        if contains_text(&document.body, keyword) {
            document.body.clone()
        } else {
            document.title.clone()
        }
    } else {
        document.body.clone()
    };

    text.chars().take(80).collect()
}

fn build_graph_excerpt(
    node: &NoNameKnowledgeGraphNode,
    nodes: &[NoNameKnowledgeGraphNode],
    edges: &[NoNameKnowledgeGraphEdge],
) -> String {
    let mut excerpt = node.body.chars().take(80).collect::<String>();
    let links = edges
        .iter()
        .filter(|edge| edge.from == node.node_id || edge.to == node.node_id)
        .take(3)
        .filter_map(|edge| {
            let neighbor_id = if edge.from == node.node_id { &edge.to } else { &edge.from };
            let neighbor = nodes.iter().find(|item| &item.node_id == neighbor_id)?;
            Some(format!("{}:{}", edge.relation, neighbor.label))
        })
        .collect::<Vec<_>>();
    if !links.is_empty() {
        excerpt.push_str(" | links=");
        excerpt.push_str(&links.join(","));
    }
    excerpt
}

fn contains_text(content: &str, keyword: &str) -> bool {
    content.to_lowercase().contains(&keyword.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_provider_returns_ranked_snippets() {
        let provider = InMemoryKnowledgeProvider::new(vec![
            NoNameKnowledgeDocument {
                document_id: "doc-1".to_string(),
                title: "青云山门志".to_string(),
                body: "山门法阵由七位长老轮值维护。".to_string(),
                tags: vec!["山门".to_string(), "法阵".to_string()],
                source: "lore".to_string(),
            },
            NoNameKnowledgeDocument {
                document_id: "doc-2".to_string(),
                title: "外门守则".to_string(),
                body: "外门弟子需每日巡山。".to_string(),
                tags: vec!["外门".to_string()],
                source: "rules".to_string(),
            },
        ]);

        let snippets = provider.retrieve(&NoNameKnowledgeQuery {
            keyword: Some("山门".to_string()),
            tags: vec!["法阵".to_string()],
            sources: vec![],
            min_score: None,
            limit: 5,
        });

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].document_id, "doc-1");
        assert_eq!(snippets[0].provider_id, "in_memory_lore");
    }

    #[test]
    fn graph_provider_recalls_related_nodes_by_relation() {
        let provider = GraphKnowledgeProvider::new(
            vec![
                NoNameKnowledgeGraphNode {
                    node_id: "gate".to_string(),
                    label: "Qingyun Gate".to_string(),
                    body: "The mountain gate is protected by an old array.".to_string(),
                    tags: vec!["sect".to_string(), "array".to_string()],
                    source: "graph-demo".to_string(),
                },
                NoNameKnowledgeGraphNode {
                    node_id: "elder".to_string(),
                    label: "Elder Qinghe".to_string(),
                    body: "The elder watches the outer gate.".to_string(),
                    tags: vec!["npc".to_string()],
                    source: "graph-demo".to_string(),
                },
            ],
            vec![NoNameKnowledgeGraphEdge {
                from: "elder".to_string(),
                to: "gate".to_string(),
                relation: "guards".to_string(),
                weight: 3,
            }],
        );

        let snippets = provider.retrieve(&NoNameKnowledgeQuery {
            keyword: Some("guards".to_string()),
            tags: vec![],
            sources: vec![],
            min_score: None,
            limit: 5,
        });

        assert_eq!(snippets.len(), 2);
        assert!(snippets.iter().any(|item| item.document_id == "gate"));
        assert!(snippets.iter().any(|item| item.document_id == "elder"));
        assert!(snippets[0].excerpt.contains("links=guards:"));
    }
}
