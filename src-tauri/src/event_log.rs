use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventImportance {
    Normal,
    Important,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub importance: EventImportance,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFilter {
    pub importance: Option<EventImportance>,
    pub event_type: Option<String>,
    pub from_timestamp: Option<u64>,
    pub to_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<GameEvent>,
    next_id: u64,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_id: 1,
        }
    }

    pub fn log_event(
        &mut self,
        timestamp: u64,
        event_type: impl Into<String>,
        description: impl Into<String>,
        importance: EventImportance,
    ) -> GameEvent {
        let event = GameEvent {
            id: self.next_id,
            timestamp,
            event_type: event_type.into(),
            description: description.into(),
            importance,
        };

        self.next_id = self.next_id.saturating_add(1);
        self.events.push(event.clone());
        event
    }

    pub fn from_events(mut events: Vec<GameEvent>) -> Self {
        events.sort_by_key(|e| (e.timestamp, e.id));
        let next_id = events
            .iter()
            .map(|e| e.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        Self { events, next_id }
    }

    pub fn all_events(&self) -> &[GameEvent] {
        &self.events
    }

    pub fn query_events(&self, filter: &EventFilter) -> Vec<GameEvent> {
        self.events
            .iter()
            .filter(|event| match &filter.importance {
                Some(expected) => &event.importance == expected,
                None => true,
            })
            .filter(|event| match &filter.event_type {
                Some(expected_type) => &event.event_type == expected_type,
                None => true,
            })
            .filter(|event| match filter.from_timestamp {
                Some(from) => event.timestamp >= from,
                None => true,
            })
            .filter(|event| match filter.to_timestamp {
                Some(to) => event.timestamp <= to,
                None => true,
            })
            .cloned()
            .collect()
    }

    pub fn important_events(&self) -> Vec<GameEvent> {
        self.query_events(&EventFilter {
            importance: Some(EventImportance::Important),
            ..EventFilter::default()
        })
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event_records_data() {
        let mut log = EventLog::new();
        let event = log.log_event(
            10,
            "combat",
            "Player defeated a rogue cultivator",
            EventImportance::Important,
        );

        assert_eq!(event.id, 1);
        assert_eq!(event.timestamp, 10);
        assert_eq!(event.event_type, "combat");
        assert_eq!(event.importance, EventImportance::Important);
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_query_events_with_filters() {
        let mut log = EventLog::new();
        log.log_event(1, "cultivation", "daily training", EventImportance::Normal);
        log.log_event(2, "combat", "sect duel", EventImportance::Important);
        log.log_event(3, "dialogue", "met elder", EventImportance::Normal);

        let combat_only = log.query_events(&EventFilter {
            event_type: Some("combat".to_string()),
            ..EventFilter::default()
        });
        assert_eq!(combat_only.len(), 1);
        assert_eq!(combat_only[0].event_type, "combat");

        let important_only = log.important_events();
        assert_eq!(important_only.len(), 1);
        assert_eq!(important_only[0].importance, EventImportance::Important);

        let ranged = log.query_events(&EventFilter {
            from_timestamp: Some(2),
            to_timestamp: Some(3),
            ..EventFilter::default()
        });
        assert_eq!(ranged.len(), 2);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 9: event recording integrity
        #[test]
        fn prop_all_important_events_are_recorded(
            items in proptest::collection::vec((0u64..1000, any::<bool>()), 1..120)
        ) {
            let mut log = EventLog::new();
            let mut expected_important = 0usize;

            for (idx, (timestamp, is_important)) in items.iter().enumerate() {
                let importance = if *is_important {
                    expected_important += 1;
                    EventImportance::Important
                } else {
                    EventImportance::Normal
                };

                log.log_event(
                    *timestamp,
                    format!("type_{}", idx % 5),
                    format!("event_{}", idx),
                    importance,
                );
            }

            let important = log.important_events();
            prop_assert_eq!(important.len(), expected_important);
            prop_assert!(important.iter().all(|e| e.importance == EventImportance::Important));
        }
    }
}
