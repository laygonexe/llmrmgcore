use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};
use regex::Regex;
use anyhow::{Result, anyhow};

// ---------- Core Primitives (from previous spec) ----------

pub type NodeId = String;
pub type EdgeId = String;
pub type TypeName = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Obj(BTreeMap<String, Value>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: TypeName,
    pub attrs: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub edge_type: TypeName,
    pub src: NodeId,
    pub dst: NodeId,
    pub attrs: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub graph: Graph,
    pub metadata: SnapshotMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub revision: String,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AttrOp {
    Eq { value: Value },
    Neq { value: Value },
    Lt { value: Value },
    Lte { value: Value },
    Gt { value: Value },
    Gte { value: Value },
    Regex { pattern: String },
    In { values: Vec<Value> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePattern {
    pub var: String,
    pub node_type: Option<TypeName>,
    pub attr_constraints: BTreeMap<String, Vec<AttrOp>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgePattern {
    pub var: String,
    pub edge_type: Option<TypeName>,
    pub src_var: String,
    pub dst_var: String,
    pub attr_constraints: BTreeMap<String, Vec<AttrOp>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StructuralConstraint {
    DistinctNodes { vars: Vec<String> },
    DistinctEdges { vars: Vec<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphPattern {
    pub nodes: Vec<NodePattern>,
    pub edges: Vec<EdgePattern>,
    pub constraints: Vec<StructuralConstraint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Match {
    pub node_bindings: BTreeMap<String, NodeId>,
    pub edge_bindings: BTreeMap<String, EdgeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub id: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DpoRule {
    pub metadata: RuleMetadata,
    pub left: GraphPattern,
    pub interface: GraphPattern,
    pub right: Graph,
}

// ---------- Domain Enums + Helpers ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Thread,
    Turn,
    Message,
    Actor,
    Concept,
    Decision,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    HasTurn,
    HasMessage,
    AuthoredBy,
    RespondsTo,
    Mentions,
    RelatesTo,
    Decides,
    BlockedBy,
    AppliesTo,
    CreatesTask,
}

impl NodeType {
    pub fn from_str(s: &str) -> Option<Self> {
        use NodeType::*;
        match s {
            "Thread"   => Some(Thread),
            "Turn"     => Some(Turn),
            "Message"  => Some(Message),
            "Actor"    => Some(Actor),
            "Concept"  => Some(Concept),
            "Decision" => Some(Decision),
            "Task"     => Some(Task),
            _ => None,
        }
    }
}

impl EdgeType {
    pub fn from_str(s: &str) -> Option<Self> {
        use EdgeType::*;
        match s {
            "HAS_TURN"      => Some(HasTurn),
            "HAS_MESSAGE"   => Some(HasMessage),
            "AUTHORED_BY"   => Some(AuthoredBy),
            "RESPONDS_TO"   => Some(RespondsTo),
            "MENTIONS"      => Some(Mentions),
            "RELATES_TO"    => Some(RelatesTo),
            "DECIDES"       => Some(Decides),
            "BLOCKED_BY"    => Some(BlockedBy),
            "APPLIES_TO"    => Some(AppliesTo),
            "CREATES_TASK"  => Some(CreatesTask),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        use EdgeType::*;
        match self {
            HasTurn     => "HAS_TURN",
            HasMessage  => "HAS_MESSAGE",
            AuthoredBy  => "AUTHORED_BY",
            RespondsTo  => "RESPONDS_TO",
            Mentions    => "MENTIONS",
            RelatesTo   => "RELATES_TO",
            Decides     => "DECIDES",
            BlockedBy   => "BLOCKED_BY",
            AppliesTo   => "APPLIES_TO",
            CreatesTask => "CREATES_TASK",
        }
    }
}

// ---------- InvariantResult & ValidationReport ----------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub is_confluent: bool,
    pub schema_valid: bool,
    pub invariants: Vec<InvariantResult>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationReport {
    pub fn from_invariants(invs: Vec<InvariantResult>) -> Self {
        let mut errors = Vec::new();
        for inv in &invs {
            if !inv.passed {
                errors.push(format!("Invariant '{}' failed: {}", inv.name, inv.message));
            }
        }

        let is_valid = errors.is_empty();
        ValidationReport {
            is_valid,
            is_confluent: true, // defer DPO confluence checks to v2
            schema_valid: is_valid,
            invariants: invs,
            warnings: Vec::new(),
            errors,
        }
    }
}

// ---------- Invariant Functions ----------

fn check_no_orphan_messages(graph: &Graph) -> InvariantResult {
    let name = "no_orphan_messages".to_string();

    let mut has_message_count: HashMap<&str, usize> = HashMap::new();
    let mut authored_by_count: HashMap<&str, usize> = HashMap::new();

    for e in &graph.edges {
        let et = match EdgeType::from_str(&e.edge_type) {
            Some(t) => t,
            None => continue,
        };

        match et {
            EdgeType::HasMessage => {
                *has_message_count.entry(e.dst.as_str()).or_insert(0) += 1;
            }
            EdgeType::AuthoredBy => {
                *authored_by_count.entry(e.src.as_str()).or_insert(0) += 1;
            }
            _ => {}
        }
    }

    let mut problems = Vec::new();

    for n in &graph.nodes {
        if NodeType::from_str(&n.node_type) != Some(NodeType::Message) {
            continue;
        }

        let msg_id = n.id.as_str();
        let hm = has_message_count.get(msg_id).copied().unwrap_or(0);
        let ab = authored_by_count.get(msg_id).copied().unwrap_or(0);

        if hm != 1 || ab != 1 {
            problems.push(format!(
                "Message {} has {} HAS_MESSAGE and {} AUTHORED_BY edges (expected 1 each)",
                msg_id, hm, ab
            ));
        }
    }

    if problems.is_empty() {
        InvariantResult {
            name,
            passed: true,
            message: "All messages have exactly one HAS_MESSAGE and one AUTHORED_BY.".into(),
        }
    } else {
        InvariantResult {
            name,
            passed: false,
            message: problems.join("; "),
        }
    }
}

fn check_no_assistant_pii_leak(graph: &Graph) -> InvariantResult {
    let name = "no_assistant_pii_leak".to_string();

    let email_re = Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();
    let phone_re = Regex::new(r"(\+?\d{1,3}[-.\s]?)?(\(?\d{3}\)?[-.\s]?)\d{3}[-.\s]?\d{4}").unwrap();

    let mut problems = Vec::new();

    for n in &graph.nodes {
        if NodeType::from_str(&n.node_type) != Some(NodeType::Message) {
            continue;
        }

        let author = n.attrs.get("author");
        let content = n.attrs.get("content");

        let is_assistant = matches!(author, Some(Value::Str(s)) if s == "assistant");

        if !is_assistant {
            continue;
        }

        let text = match content {
            Some(Value::Str(s)) => s,
            _ => continue,
        };

        if email_re.is_match(text) || phone_re.is_match(text) {
            problems.push(format!(
                "Assistant message {} appears to contain PII.",
                n.id
            ));
        }
    }

    if problems.is_empty() {
        InvariantResult {
            name,
            passed: true,
            message: "No assistant messages contain obvious PII.".into(),
        }
    } else {
        InvariantResult {
            name,
            passed: false,
            message: problems.join("; "),
        }
    }
}

fn check_well_typed_edges(graph: &Graph) -> InvariantResult {
    let name = "well_typed_edges".to_string();

    let mut node_types: HashMap<&str, NodeType> = HashMap::new();
    for n in &graph.nodes {
        if let Some(nt) = NodeType::from_str(&n.node_type) {
            node_types.insert(&n.id, nt);
        }
    }

    let mut problems = Vec::new();

    for e in &graph.edges {
        let et = match EdgeType::from_str(&e.edge_type) {
            Some(et) => et,
            None => {
                problems.push(format!("Unknown edge type '{}' on edge {}", e.edge_type, e.id));
                continue;
            }
        };

        let src_ty = match node_types.get(e.src.as_str()) {
            Some(ty) => ty,
            None => {
                problems.push(format!(
                    "Edge {} has unknown src node {}",
                    e.id, e.src
                ));
                continue;
            }
        };
        let dst_ty = match node_types.get(e.dst.as_str()) {
            Some(ty) => ty,
            None => {
                problems.push(format!(
                    "Edge {} has unknown dst node {}",
                    e.id, e.dst
                ));
                continue;
            }
        };

        use EdgeType::*;
        use NodeType::*;

        let ok = match (et, src_ty, dst_ty) {
            (HasTurn,    Thread,  Turn)     => true,
            (HasMessage, Turn,    Message)  => true,
            (AuthoredBy, Message, Actor)    => true,
            (RespondsTo, Message, Message)  => true,
            (Mentions,   Message, Concept)  => true,
            (RelatesTo,  Concept, Concept)  => true,
            (Decides,    Decision, Concept) => true,
            (Decides,    Decision, Task)    => true,
            (BlockedBy,  Task,    Task)     => true,
            (BlockedBy,  Task,    Concept)  => true,
            (AppliesTo,  Decision, Thread)  => true,
            (CreatesTask, Message, Task)    => true,
            _ => false,
        };

        if !ok {
            problems.push(format!(
                "Edge {} of type {} has illegal src/dst types: {:?} -> {:?}",
                e.id,
                et.as_str(),
                src_ty,
                dst_ty
            ));
        }
    }

    if problems.is_empty() {
        InvariantResult {
            name,
            passed: true,
            message: "All edges respect allowed src/dst types.".into(),
        }
    } else {
        InvariantResult {
            name,
            passed: false,
            message: problems.join("; "),
        }
    }
}

fn check_immutable_history(rule: &DpoRule) -> InvariantResult {
    let name = "immutable_history".to_string();

    let mut message_vars_in_left: HashSet<&str> = HashSet::new();
    for np in &rule.left.nodes {
        let is_message = match &np.node_type {
            Some(t) if t == "Message" => true,
            _ => false,
        };
        if is_message {
            message_vars_in_left.insert(np.var.as_str());
        }
    }

    if message_vars_in_left.is_empty() {
        return InvariantResult {
            name,
            passed: true,
            message: "Rule does not touch any Message nodes in L.".into(),
        };
    }

    let interface_vars: HashSet<&str> =
        rule.interface.nodes.iter().map(|n| n.var.as_str()).collect();

    let mut vars_in_right: HashSet<String> = HashSet::new();
    for n in &rule.right.nodes {
        if let Some(stripped) = n.id.strip_prefix("var:") {
            vars_in_right.insert(stripped.to_string());
        }
    }

    let mut problems = Vec::new();

    for v in &message_vars_in_left {
        if !interface_vars.contains(v) {
            problems.push(format!(
                "Message var '{}' appears in L but not in interface K (would allow deletion).",
                v
            ));
        }
        if !vars_in_right.contains(*v) {
            problems.push(format!(
                "Message var '{}' appears in L but no corresponding 'var:{}' node in R (would delete message).",
                v, v
            ));
        }
    }

    if problems.is_empty() {
        InvariantResult {
            name,
            passed: true,
            message: "All Messages in L are preserved via K and R; no deletions.".into(),
        }
    } else {
        InvariantResult {
            name,
            passed: false,
            message: problems.join("; "),
        }
    }
}

// ---------- RmgCore & LlmOracle traits ----------

pub trait RmgCore {
    fn query(&self, pattern: &GraphPattern) -> Result<Vec<Match>>;
    fn snapshot(&self) -> GraphSnapshot;
    fn validate(&self, rule: &DpoRule) -> Result<ValidationReport>;
    fn apply(&mut self, rule: &DpoRule) -> Result<ExecutionProof>;
}

pub trait LlmOracle {
    fn propose_query(&self, natural_language: &str) -> Result<GraphPattern>;
    fn propose_rule(
        &self,
        natural_language: &str,
        context: &GraphSnapshot,
    ) -> Result<DpoRule>;
    fn explain_proof(&self, proof: &ExecutionProof) -> Result<String>;
    fn refine_with_feedback(
        &self,
        command: &str,
        context: &GraphSnapshot,
        previous_proposal: &DpoRule,
        report: &ValidationReport,
    ) -> Result<DpoRule>;
}

// ---------- InMemoryCore Implementation ----------

#[derive(Clone, Debug)]
pub struct InMemoryCore {
    pub snapshot: GraphSnapshot,
    next_node_id: u64,
    next_edge_id: u64,
}

impl InMemoryCore {
    pub fn new() -> Self {
        InMemoryCore {
            snapshot: GraphSnapshot {
                graph: Graph::default(),
                metadata: SnapshotMetadata {
                    revision: "rev-0".to_string(),
                    timestamp: Utc::now(),
                    actor_id: "system".to_string(),
                    description: "Initial empty state".to_string(),
                },
            },
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    fn simulate_apply(&self, rule: &DpoRule) -> Result<Graph> {
        let mut new_graph = self.snapshot.graph.clone();
        let mut id_map: HashMap<String, String> = HashMap::new();

        // For v0, we assume the `left` and `interface` patterns are simple and mainly
        // serve to identify the nodes we're operating on. We'll find the first match
        // for any `var`s mentioned.
        for np in &rule.left.nodes {
            if let Some(node) = new_graph.nodes.iter().find(|n| n.node_type == np.node_type.as_ref().unwrap().as_str()) {
                id_map.insert(format!("var:{}", np.var), node.id.clone());
            }
        }

        // Add new nodes and map their temporary IDs
        let mut next_node_id = self.next_node_id;
        for node in &rule.right.nodes {
            if node.id.starts_with("new:") {
                let new_id = format!("n{}", next_node_id);
                next_node_id += 1;
                let mut new_node = node.clone();
                new_node.id = new_id.clone();
                id_map.insert(node.id.clone(), new_id);
                new_graph.nodes.push(new_node);
            }
        }

        // Add new edges, resolving IDs
        let mut next_edge_id = self.next_edge_id;
        for edge in &rule.right.edges {
            if edge.id.starts_with("new:") {
                let new_id = format!("e{}", next_edge_id);
                next_edge_id += 1;
                let mut new_edge = edge.clone();
                new_edge.id = new_id;
                
                if let Some(src_id) = id_map.get(&edge.src) {
                    new_edge.src = src_id.clone();
                }
                if let Some(dst_id) = id_map.get(&edge.dst) {
                    new_edge.dst = dst_id.clone();
                }
                
                new_graph.edges.push(new_edge);
            }
        }

        Ok(new_graph)
    }

    fn apply_rewrite(&mut self, rule: &DpoRule) -> Result<()> {
        let new_graph = self.simulate_apply(rule)?;
        self.snapshot.graph = new_graph;
        // In a real implementation, we'd update the next_node_id and next_edge_id fields here.
        Ok(())
    }
}

impl RmgCore for InMemoryCore {
    fn query(&self, _pattern: &GraphPattern) -> anyhow::Result<Vec<Match>> {
        unimplemented!("V0 does not implement query")
    }

    fn snapshot(&self) -> GraphSnapshot {
        self.snapshot.clone()
    }

    fn validate(&self, rule: &DpoRule) -> anyhow::Result<ValidationReport> {
        let sandbox_graph = self.simulate_apply(rule)?;

        let mut invariants = Vec::new();
        invariants.push(check_no_orphan_messages(&sandbox_graph));
        invariants.push(check_no_assistant_pii_leak(&sandbox_graph));
        invariants.push(check_well_typed_edges(&sandbox_graph));
        invariants.push(check_immutable_history(rule));

        Ok(ValidationReport::from_invariants(invariants))
    }

    fn apply(&mut self, rule: &DpoRule) -> anyhow::Result<ExecutionProof> {
        let report = self.validate(rule)?;
        if !report.is_valid {
            anyhow::bail!("Refusing to apply invalid rule: {:?}", report.errors);
        }

        let before = self.snapshot();
        self.apply_rewrite(rule)?;
        let mut after = self.snapshot();
        after.metadata.revision = format!("rev-{}", before.metadata.revision.split('-').last().unwrap_or("0").parse::<u64>().unwrap_or(0) + 1);


        let proof = ExecutionProof {
            rule_metadata: rule.metadata.clone(),
            rule_hash: "TODO-rule-hash".into(),
            before_revision: before.metadata.revision.clone(),
            after_revision: after.metadata.revision.clone(),
            before_snapshot: Some(before),
            after_snapshot: Some(after),
            ..Default::default()
        };

        Ok(proof)
    }
}

// ---------- Mock LLM Oracle Implementation ----------

pub struct MockOracle;

impl LlmOracle for MockOracle {
    fn propose_query(&self, _natural_language: &str) -> Result<GraphPattern> {
        unimplemented!("V0 does not implement propose_query")
    }

    fn propose_rule(
        &self,
        natural_language: &str,
        _context: &GraphSnapshot,
    ) -> Result<DpoRule> {
        if natural_language.to_lowercase().contains("create a task") {
            let fixed_timestamp = DateTime::parse_from_rfc3339("2025-11-15T12:00:00Z").unwrap().with_timezone(&Utc);
            // Return a hardcoded rule for creating a task
            Ok(DpoRule {
                metadata: RuleMetadata {
                    id: "rho_create_task_from_message_mock".to_string(),
                    version: "0.1.0".to_string(),
                    description: "Creates a new Task node from a user message.".to_string(),
                    author: "mock-oracle".to_string(),
                    created_at: fixed_timestamp,
                    tags: vec!["task".to_string(), "creation".to_string()],
                },
                left: GraphPattern {
                    nodes: vec![
                        NodePattern {
                            var: "msg".to_string(),
                            node_type: Some("Message".to_string()),
                            attr_constraints: BTreeMap::new(),
                        }
                    ],
                    edges: vec![],
                    constraints: vec![],
                },
                interface: GraphPattern {
                    nodes: vec![
                        NodePattern {
                            var: "msg".to_string(),
                            node_type: Some("Message".to_string()),
                            attr_constraints: BTreeMap::new(),
                        }
                    ],
                    edges: vec![],
                    constraints: vec![],
                },
                right: Graph {
                    nodes: vec![
                        Node {
                            id: "var:msg".to_string(),
                            node_type: "Message".to_string(),
                            attrs: BTreeMap::new(),
                        },
                        Node {
                            id: "new:task".to_string(),
                            node_type: "Task".to_string(),
                            attrs: {
                                let mut map = BTreeMap::new();
                                map.insert("title".to_string(), Value::Str("Write the report".to_string()));
                                map.insert("status".to_string(), Value::Str("Pending".to_string()));
                                map
                            },
                        }
                    ],
                    edges: vec![
                        Edge {
                            id: "new:edge".to_string(),
                            edge_type: "CREATES_TASK".to_string(),
                            src: "var:msg".to_string(),
                            dst: "new:task".to_string(),
                            attrs: BTreeMap::new(),
                        }
                    ],
                }
            })
        } else {
            Err(anyhow!("MockOracle doesn't know how to handle this command"))
        }
    }

    fn explain_proof(&self, _proof: &ExecutionProof) -> Result<String> {
        Ok("The rule was applied successfully.".to_string())
    }

    fn refine_with_feedback(
        &self,
        _command: &str,
        _context: &GraphSnapshot,
        previous_proposal: &DpoRule,
        _report: &ValidationReport,
    ) -> Result<DpoRule> {
        // V0: a real implementation would try to fix the rule.
        // Here, we just return the same rule, assuming it will fail again.
        Ok(previous_proposal.clone())
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionProof {
    pub rule_metadata: RuleMetadata,
    pub rule_hash: String,
    pub before_revision: String,
    pub after_revision: String,
    pub before_snapshot: Option<GraphSnapshot>,
    pub after_snapshot: Option<GraphSnapshot>,
    pub confluence_proof: Option<ConfluenceProof>,
    pub diff_summary: DiffSummary,
    pub actor_id: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DiffSummary {
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConfluenceProof {
    pub method: String,
    pub details: String,
}

impl Default for RuleMetadata {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            version: "".to_string(),
            description: "".to_string(),
            tags: vec![],
            author: "".to_string(),
            created_at: Utc::now(),
        }
    }
}