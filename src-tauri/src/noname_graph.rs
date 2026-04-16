use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{NoNameRole, NoNameTraceStage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoNameGraphNode {
    CollectTurnInput,
    BuildContextBundle,
    PlanTurn,
    PersistTrace,
}

impl NoNameGraphNode {
    pub fn to_trace_stage(self) -> NoNameTraceStage {
        match self {
            Self::CollectTurnInput => NoNameTraceStage::CollectTurnInput,
            Self::BuildContextBundle => NoNameTraceStage::BuildContextBundle,
            Self::PlanTurn => NoNameTraceStage::PlanTurn,
            Self::PersistTrace => NoNameTraceStage::PersistTrace,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameGraphExecutionResult {
    pub graph_path: Vec<NoNameTraceStage>,
}

#[derive(Debug, Default, Clone)]
pub struct NoNameGraphExecutor;

impl NoNameGraphExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn default_path() -> &'static [NoNameGraphNode] {
        &[
            NoNameGraphNode::CollectTurnInput,
            NoNameGraphNode::BuildContextBundle,
            NoNameGraphNode::PlanTurn,
            NoNameGraphNode::PersistTrace,
        ]
    }

    pub fn default_role_dispatch_order() -> &'static [NoNameRole] {
        &[
            NoNameRole::Director,
            NoNameRole::WorldCurator,
            NoNameRole::NpcIntent,
            NoNameRole::CombatNarrator,
        ]
    }

    pub fn execute_empty_turn(
        &self,
        trace: &mut NoNameTrace,
    ) -> Result<NoNameGraphExecutionResult, NoNameError> {
        let nodes = Self::default_path();
        if nodes.is_empty() {
            return Err(NoNameError::new(
                NoNameErrorKind::Runtime,
                "graph path is empty",
                "noname.graph.empty_path",
                false,
            ));
        }

        for node in nodes {
            trace.push_stage(node.to_trace_stage());
        }

        Ok(NoNameGraphExecutionResult {
            graph_path: trace.graph_path.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_types::NoNameMode;

    #[test]
    fn default_path_contains_minimum_nodes() {
        assert_eq!(
            NoNameGraphExecutor::default_path(),
            &[
                NoNameGraphNode::CollectTurnInput,
                NoNameGraphNode::BuildContextBundle,
                NoNameGraphNode::PlanTurn,
                NoNameGraphNode::PersistTrace,
            ]
        );
    }

    #[test]
    fn default_role_dispatch_order_starts_from_director() {
        assert_eq!(
            NoNameGraphExecutor::default_role_dispatch_order(),
            &[
                NoNameRole::Director,
                NoNameRole::WorldCurator,
                NoNameRole::NpcIntent,
                NoNameRole::CombatNarrator,
            ]
        );
    }

    #[test]
    fn executor_records_graph_path_on_trace() {
        let executor = NoNameGraphExecutor::new();
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        let result = executor
            .execute_empty_turn(&mut trace)
            .expect("graph execution should succeed");

        assert_eq!(result.graph_path.len(), 4);
        assert_eq!(trace.graph_path, result.graph_path);
    }
}
