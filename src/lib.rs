//! llmrmgcore: An adapter for controlling a deterministic RMG Core with a stochastic LLM Oracle.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rmg_core::{
    build_motion_demo_engine, motion_rule, Engine, GraphStore, NodeId, NodeRecord, RewriteRule,
    Snapshot, TypeId, MOTION_RULE_NAME,
};
use std::collections::BTreeMap;

// --- Public Traits (The "Ports" of our Hexagonal Architecture) ---

/// The RmgCore trait defines the interface for a deterministic graph engine.
/// This is the contract that the core implementation must adhere to.
pub trait RmgCore {
    /// Applies a named rule to a specific node (the scope).
    fn apply(&mut self, rule_name: &str, scope: &NodeId) -> Result<Snapshot>;

    /// Returns an immutable snapshot of the current graph state.
    fn snapshot(&self) -> Snapshot;

    /// Registers a new rewrite rule with the engine.
    fn register_rule(&mut self, rule: RewriteRule) -> Result<()>;

    // Helper to get the underlying graph store for inspection.
    fn store(&self) -> &GraphStore;
}

/// The LlmOracle trait defines the interface for the stochastic "proposer".
/// It translates natural language into formal, executable commands for the RmgCore.
pub trait LlmOracle {
    /// Proposes a rule to execute based on a natural language command.
    fn propose_rule(&self, natural_language: &str) -> Result<(String, NodeId)>;
}

// --- RmgCoreAdapter Implementation ---

/// An adapter that wraps the `rmg_core::Engine` and implements our `RmgCore` trait.
pub struct RmgCoreAdapter {
    engine: Engine,
}

impl RmgCoreAdapter {
    /// Creates a new adapter, pre-registering a set of domain-specific rules.
    pub fn new() -> Self {
        let mut engine = build_motion_demo_engine(); // Starts with a world-root and motion rule
        engine
            .register_rule(create_task_rule())
            .expect("should register create_task_rule");
        Self { engine }
    }
}

impl Default for RmgCoreAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RmgCore for RmgCoreAdapter {
    fn apply(&mut self, rule_name: &str, scope: &NodeId) -> Result<Snapshot> {
        let tx = self.engine.begin();
        self.engine
            .apply(tx, rule_name, scope)
            .map_err(|e| anyhow!("Engine apply failed: {:?}", e))?;
        self.engine
            .commit(tx)
            .map_err(|e| anyhow!("Engine commit failed: {:?}", e))
    }

    fn snapshot(&self) -> Snapshot {
        self.engine.snapshot()
    }

    fn register_rule(&mut self, rule: RewriteRule) -> Result<()> {
        self.engine
            .register_rule(rule)
            .map_err(|e| anyhow!("Failed to register rule: {:?}", e))
    }

    fn store(&self) -> &GraphStore {
        self.engine.store()
    }
}

// --- Domain-Specific Rules ("Conversation & Decision Log" Domain) ---

pub const CREATE_TASK_RULE_NAME: &str = "conversation/create_task";

/// A rewrite rule that creates a `Task` node and links it to a `Message` node.
pub fn create_task_rule() -> RewriteRule {
    // A matcher function that checks if the rule can be applied to the given node.
    fn matcher(store: &GraphStore, scope: &NodeId) -> bool {
        store
            .node(scope)
            .map_or(false, |n| n.ty == rmg_core::make_type_id("Message"))
    }

    // An executor function that performs the graph modification.
    fn executor(store: &mut GraphStore, scope: &NodeId) {
        let task_id = rmg_core::make_node_id("task-from-message");
        let task_type = rmg_core::make_type_id("Task");
        let edge_type = rmg_core::make_type_id("CREATES_TASK");

        // Insert the new Task node
        store.insert_node(
            task_id,
            NodeRecord {
                ty: task_type,
                payload: Some(b"A new task".to_vec()), // Simple payload
            },
        );

        // Insert the new edge connecting the message to the task
        store.insert_edge(edge_type, *scope, task_id, None);
    }

    RewriteRule {
        id: rmg_core::make_type_id(CREATE_TASK_RULE_NAME), // Deterministic ID
        name: CREATE_TASK_RULE_NAME,
        left: rmg_core::PatternGraph { nodes: vec![] }, // Matcher is programmatic
        matcher,
        executor,
        // For this simple additive rule, the footprint is straightforward.
        compute_footprint: |_, scope| {
            let mut n_read = rmg_core::IdSet::default();
            n_read.insert_node(scope);
            rmg_core::Footprint {
                n_read,
                ..Default::default()
            }
        },
        factor_mask: 0,
        conflict_policy: rmg_core::ConflictPolicy::Abort,
        join_fn: None,
    }
}

// --- Mock LLM Oracle Implementation ---

/// A mock implementation of the `LlmOracle`.
pub struct MockOracle {
    /// The ID of the message to target. In a real system, this would be determined dynamically.
    pub target_message_id: NodeId,
}

impl LlmOracle for MockOracle {
    fn propose_rule(&self, natural_language: &str) -> Result<(String, NodeId)> {
        if natural_language.to_lowercase().contains("create a task") {
            // The oracle's job is to map NL to a specific, registered rule and a scope.
            Ok((CREATE_TASK_RULE_NAME.to_string(), self.target_message_id))
        } else if natural_language.to_lowercase().contains("move") {
            Ok((MOTION_RULE_NAME.to_string(), self.target_message_id))
        } else {
            Err(anyhow!(
                "MockOracle doesn't know how to handle this command"
            ))
        }
    }
}
