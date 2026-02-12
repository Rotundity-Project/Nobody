use crate::event_log::GameEvent;
use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub index: u32,
    pub title: String,
    pub content: String,
    pub source_event_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Novel {
    pub title: String,
    pub chapters: Vec<Chapter>,
    pub total_events: usize,
}

pub struct NovelGenerator {
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
    chapter_event_batch_size: usize,
}

impl NovelGenerator {
    pub fn new() -> Self {
        Self {
            llm_service: Self::initialize_llm_service_from_env(),
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
            chapter_event_batch_size: 8,
        }
    }

    fn initialize_llm_service_from_env() -> Option<LLMService> {
        let cfg = resolve_llm_config()?;
        LLMService::new(cfg).ok()
    }

    pub async fn generate_novel(
        &self,
        title: impl Into<String>,
        events: &[GameEvent],
    ) -> Result<Novel, String> {
        let title = title.into();
        let mut ordered_events = events.to_vec();
        ordered_events.sort_by_key(|e| (e.timestamp, e.id));

        if ordered_events.is_empty() {
            return Ok(Novel {
                title,
                chapters: vec![Chapter {
                    index: 1,
                    title: "第1章：静水初澜".to_string(),
                    content: "尚无重大事件发生，你的修行旅程正等待展开。".to_string(),
                    source_event_ids: Vec::new(),
                }],
                total_events: 0,
            });
        }

        let mut chapters = Vec::new();
        for (idx, chunk) in ordered_events
            .chunks(self.chapter_event_batch_size.max(1))
            .enumerate()
        {
            let chapter = self.generate_chapter((idx + 1) as u32, chunk).await?;
            chapters.push(chapter);
        }

        Ok(Novel {
            title,
            chapters,
            total_events: ordered_events.len(),
        })
    }

    pub async fn generate_chapter(
        &self,
        chapter_index: u32,
        events: &[GameEvent],
    ) -> Result<Chapter, String> {
        let source_event_ids = events.iter().map(|e| e.id).collect::<Vec<u64>>();
        let title = format!("第{}章：命途流转", chapter_index);

        if let Some(content) = self.generate_chapter_with_llm(chapter_index, events).await {
            return Ok(Chapter {
                index: chapter_index,
                title,
                content,
                source_event_ids,
            });
        }

        Ok(Chapter {
            index: chapter_index,
            title,
            content: self.generate_chapter_fallback(events),
            source_event_ids,
        })
    }

    async fn generate_chapter_with_llm(
        &self,
        chapter_index: u32,
        events: &[GameEvent],
    ) -> Option<String> {
        if cfg!(test) {
            return None;
        }

        let llm_service = self.llm_service.as_ref()?;
        let event_lines = events
            .iter()
            .map(|e| format!("[t={}] {}: {}", e.timestamp, e.event_type, e.description))
            .collect::<Vec<String>>()
            .join("\n");

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some(format!("请根据时间线事件生成第 {} 章小说正文", chapter_index)),
                location: None,
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: vec![event_lines],
                world_setting_summary: Some(
                    "修仙小说文风，保留事件顺序，章节结尾留出后续发展空间".to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec!["保持时间顺序，不可颠倒因果".to_string()],
                world_rules: vec![
                    "仅输出中文纯文本".to_string(),
                    "字数控制在 300-700 字".to_string(),
                    "叙事风格接近连载网文".to_string(),
                ],
                output_schema_hint: None,
            },
            600,
        );

        let response = llm_service
            .generate(LLMRequest {
                prompt,
                max_tokens: Some(500),
                temperature: Some(0.8),
            })
            .await
            .ok()?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        let text = response.text.trim();
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }

    fn generate_chapter_fallback(&self, events: &[GameEvent]) -> String {
        if events.is_empty() {
            return "这一章尚未掀起波澜，主角在平静中积蓄力量。".to_string();
        }

        let mut lines = Vec::with_capacity(events.len() + 1);
        lines.push("修行之路在以下片段中继续推进：".to_string());
        for event in events {
            lines.push(format!(
                "第{}日：{}（{}）",
                event.timestamp, event.description, event.event_type
            ));
        }
        lines.push("故事尚未结束，你的下一次选择将决定后续走向。".to_string());
        lines.join("\n")
    }

    pub fn export_to_file(&self, novel: &Novel, file_path: impl AsRef<Path>) -> Result<(), String> {
        let path = file_path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut content = String::new();
        content.push_str(&format!("{}\n\n", novel.title));
        content.push_str(&format!("事件总数：{}\n\n", novel.total_events));

        for chapter in &novel.chapters {
            content.push_str(&format!("第{}章 - {}\n", chapter.index, chapter.title));
            content.push_str(&chapter.content);
            content.push_str("\n\n");
        }

        std::fs::write(path, content).map_err(|e| e.to_string())
    }
}

impl Default for NovelGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::EventImportance;

    fn test_event(id: u64, timestamp: u64, event_type: &str, description: &str) -> GameEvent {
        GameEvent {
            id,
            timestamp,
            event_type: event_type.to_string(),
            description: description.to_string(),
            importance: EventImportance::Normal,
        }
    }

    #[tokio::test]
    async fn test_generate_novel_creates_chapters() {
        let generator = NovelGenerator::new();
        let events = vec![
            test_event(1, 1, "cultivation", "Player cultivated at dawn"),
            test_event(2, 2, "combat", "Player won a duel"),
        ];

        let novel = generator
            .generate_novel("Road to Immortality", &events)
            .await
            .unwrap();

        assert_eq!(novel.title, "Road to Immortality");
        assert_eq!(novel.total_events, 2);
        assert!(!novel.chapters.is_empty());
    }

    #[tokio::test]
    async fn test_generate_chapter_contains_event_content() {
        let generator = NovelGenerator::new();
        let events = vec![
            test_event(1, 1, "dialogue", "An elder shared a warning"),
            test_event(2, 2, "breakthrough", "Realm barrier started to shake"),
        ];

        let chapter = generator.generate_chapter(1, &events).await.unwrap();
        assert!(!chapter.content.is_empty());
        assert_eq!(chapter.source_event_ids, vec![1, 2]);
    }

    #[test]
    fn test_export_to_file_creates_txt() {
        let generator = NovelGenerator::new();
        let novel = Novel {
            title: "Export Test".to_string(),
            chapters: vec![Chapter {
                index: 1,
                title: "Beginning".to_string(),
                content: "A quiet dawn over the sect.".to_string(),
                source_event_ids: vec![1],
            }],
            total_events: 1,
        };

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("novel.txt");
        let result = generator.export_to_file(&novel, &output);
        assert!(result.is_ok());
        assert!(output.exists());

        let text = std::fs::read_to_string(output).unwrap();
        assert!(text.contains("Export Test"));
        assert!(text.contains("Beginning"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::event_log::EventImportance;
    use proptest::prelude::*;

    fn build_event(id: u64, timestamp: u64, desc: String) -> GameEvent {
        GameEvent {
            id,
            timestamp,
            event_type: "event".to_string(),
            description: desc,
            importance: EventImportance::Normal,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 10: novel generation availability
        #[test]
        fn prop_any_history_can_generate_novel(
            descriptions in proptest::collection::vec("[a-zA-Z0-9 ]{1,40}", 0..60)
        ) {
            let events = descriptions
                .iter()
                .enumerate()
                .map(|(i, d)| build_event((i + 1) as u64, (i + 1) as u64, d.clone()))
                .collect::<Vec<GameEvent>>();

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let generator = NovelGenerator::new();
            let result = runtime.block_on(generator.generate_novel("Test", &events));

            prop_assert!(result.is_ok());
            let novel = result.unwrap();
            prop_assert!(!novel.chapters.is_empty());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 12: novel keeps event chronology
        #[test]
        fn prop_novel_preserves_event_order(
            timestamps in proptest::collection::vec(1u64..2000u64, 1..50)
        ) {
            let mut sorted = timestamps.clone();
            sorted.sort_unstable();

            let events = sorted
                .iter()
                .enumerate()
                .map(|(i, ts)| build_event((i + 1) as u64, *ts, format!("event_{}", i)))
                .collect::<Vec<GameEvent>>();

            let runtime = tokio::runtime::Runtime::new().unwrap();
            let generator = NovelGenerator::new();
            let novel = runtime
                .block_on(generator.generate_novel("Timeline", &events))
                .unwrap();

            let mut all_ids = Vec::new();
            for chapter in &novel.chapters {
                all_ids.extend(chapter.source_event_ids.iter().copied());
            }

            let expected_ids = events.iter().map(|e| e.id).collect::<Vec<u64>>();
            prop_assert_eq!(all_ids, expected_ids);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 11: novel export capability
        #[test]
        fn prop_any_novel_can_be_exported(
            title in "[A-Za-z0-9 ]{1,40}",
            chapter_count in 1usize..6,
            body in "[A-Za-z0-9 ,.]{1,120}"
        ) {
            let chapters = (0..chapter_count)
                .map(|idx| Chapter {
                    index: (idx + 1) as u32,
                    title: format!("Chapter {}", idx + 1),
                    content: format!("{} {}", body, idx),
                    source_event_ids: vec![idx as u64 + 1],
                })
                .collect::<Vec<Chapter>>();

            let novel = Novel {
                title: title.clone(),
                chapters,
                total_events: chapter_count,
            };

            let dir = tempfile::tempdir().unwrap();
            let output = dir.path().join("exported_novel.txt");
            let generator = NovelGenerator::new();
            let result = generator.export_to_file(&novel, &output);

            prop_assert!(result.is_ok());
            prop_assert!(output.exists());
            let content = std::fs::read_to_string(output).unwrap();
            prop_assert!(content.contains(&title));
        }
    }
}

