//! Native (Rust) port of the SkillOpt optimization loop
//! (Microsoft Research — <https://microsoft.github.io/SkillOpt/>).
//!
//! SkillOpt "treats a compact natural-language skill document as the trainable
//! state of a frozen language agent": a separate optimizer model turns scored
//! rollouts into bounded add/delete/replace edits on a single skill document,
//! and an edit is accepted only when it strictly improves a held-out
//! validation score. The published machinery — a textual learning-rate budget,
//! a rejected-edit buffer, success/failure minibatch separation, and an
//! epoch-wise slow/meta update — is reproduced here deterministically.
//!
//! What is native here is the *control plane*, not the LLM. The reflect step
//! that an optimizer model performs (reading rollouts and proposing edits) is
//! modeled deterministically: the optimizer ingests offline scored rollouts
//! (the `tasks`) and proposes the recurring procedures missing from the
//! failures. Swapping in a real optimizer model would replace only that
//! recurrence heuristic — the loop, budget, gate, rejected buffer, and slow
//! update stay identical. Output is the single deployable markdown skill plus
//! a per-step training history, exactly the SkillOpt deliverable.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde_json::{json, Value};

/// Weight of partial required-key coverage in the held-out validation score.
/// Task pass-rate dominates; coverage is a smooth tie-breaker so the gate can
/// reward progress toward completing a validation task before it fully passes.
pub const COVERAGE_WEIGHT: f64 = 1e-3;

/// Which split a rollout belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    Train,
    Val,
}

impl Split {
    fn from_str(s: &str) -> Split {
        match s {
            "val" | "validation" | "valid" | "held-out" | "heldout" => Split::Val,
            _ => Split::Train,
        }
    }
}

/// A single scored rollout: the outcome of running the frozen target model on
/// one task under the current skill. `requires` lists the procedure keys the
/// task needs the skill to encode in order to succeed.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub split: Split,
    pub success: bool,
    pub requires: Vec<String>,
}

impl Task {
    fn from_value(idx: usize, v: &Value) -> Task {
        let id = v
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("task-{idx}"));
        let split = v
            .get("split")
            .and_then(Value::as_str)
            .map(Split::from_str)
            .unwrap_or(Split::Train);
        // `success` may be given directly, or derived from a numeric `score`.
        let success = match v.get("success").and_then(Value::as_bool) {
            Some(b) => b,
            None => v
                .get("score")
                .and_then(Value::as_f64)
                .map(|s| s >= 0.5)
                .unwrap_or(false),
        };
        let requires = v
            .get("requires")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Task {
            id,
            split,
            success,
            requires,
        }
    }
}

/// A catalog entry maps a procedure key to its rendered bullet text and, for a
/// `replace` edit, the key it supersedes.
#[derive(Debug, Clone, Default)]
pub struct CatalogEntry {
    pub text: Option<String>,
    pub supersedes: Option<String>,
}

/// Key -> catalog entry. Drives rendering and the `replace` operation.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub entries: BTreeMap<String, CatalogEntry>,
}

impl Catalog {
    fn from_value(v: Option<&Value>) -> Catalog {
        let mut entries = BTreeMap::new();
        if let Some(Value::Object(map)) = v {
            for (k, val) in map {
                let entry = match val {
                    // Shorthand: "key": "bullet text".
                    Value::String(s) => CatalogEntry {
                        text: Some(s.clone()),
                        supersedes: None,
                    },
                    Value::Object(_) => CatalogEntry {
                        text: val.get("text").and_then(Value::as_str).map(str::to_string),
                        supersedes: val
                            .get("supersedes")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    },
                    _ => CatalogEntry::default(),
                };
                entries.insert(k.clone(), entry);
            }
        }
        Catalog { entries }
    }

    fn text_for(&self, key: &str) -> String {
        self.entries
            .get(key)
            .and_then(|e| e.text.clone())
            .unwrap_or_else(|| key.to_string())
    }

    fn supersedes(&self, key: &str) -> Option<&String> {
        self.entries.get(key).and_then(|e| e.supersedes.as_ref())
    }
}

/// A frozen section of the skill document, rendered verbatim above the
/// learned-procedure list.
#[derive(Debug, Clone)]
pub struct Section {
    pub heading: Option<String>,
    pub text: String,
}

/// The skill document: a frozen base plus the trainable ordered set of
/// procedure keys (`active`).
#[derive(Debug, Clone)]
pub struct Skill {
    pub title: String,
    pub sections: Vec<Section>,
    pub active: Vec<String>,
}

impl Skill {
    fn from_value(v: Option<&Value>) -> Skill {
        let title = v
            .and_then(|s| s.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Skill")
            .to_string();
        let sections = v
            .and_then(|s| s.get("sections"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .map(|sec| Section {
                        heading: sec
                            .get("heading")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        text: sec
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let active = v
            .and_then(|s| s.get("lessons").or_else(|| s.get("active")))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Skill {
            title,
            sections,
            active,
        }
    }
}

/// A full SkillOpt run input: the starting skill, the scored rollouts, and the
/// procedure catalog.
#[derive(Debug, Clone)]
pub struct Dataset {
    pub skill: Skill,
    pub tasks: Vec<Task>,
    pub catalog: Catalog,
}

impl Dataset {
    pub fn from_value(payload: &Value) -> Dataset {
        let skill = Skill::from_value(payload.get("skill"));
        let tasks = payload
            .get("tasks")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .enumerate()
                    .map(|(i, v)| Task::from_value(i, v))
                    .collect()
            })
            .unwrap_or_default();
        let catalog = Catalog::from_value(payload.get("catalog"));
        Dataset {
            skill,
            tasks,
            catalog,
        }
    }
}

/// Hyperparameters. The edit budget is the "textual learning rate": the cap on
/// add/replace operations carried by a single candidate.
#[derive(Debug, Clone)]
pub struct OptConfig {
    pub epochs: usize,
    /// Minibatch size over train rollouts; `0` means one batch of all train.
    pub batch_size: usize,
    /// Textual learning rate: max add/replace edits per candidate.
    pub edit_budget: usize,
    /// Min validation improvement required to accept an add/replace candidate.
    pub gate_margin: f64,
    /// Max stale-key deletes proposed per epoch (slow/meta update).
    pub slow_cap: usize,
}

impl Default for OptConfig {
    fn default() -> Self {
        OptConfig {
            epochs: 3,
            batch_size: 0,
            edit_budget: 3,
            gate_margin: 0.0,
            slow_cap: 1,
        }
    }
}

/// The kind of bounded edit applied to the skill document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Add,
    Delete,
    Replace,
}

impl EditKind {
    fn as_str(&self) -> &'static str {
        match self {
            EditKind::Add => "add",
            EditKind::Delete => "delete",
            EditKind::Replace => "replace",
        }
    }
}

/// A single proposed edit on a procedure key.
#[derive(Debug, Clone)]
pub struct Edit {
    pub kind: EditKind,
    pub key: String,
    /// For `replace`, the superseded key that is removed.
    pub replaced: Option<String>,
}

impl Edit {
    fn to_value(&self) -> Value {
        json!({
            "op": self.kind.as_str(),
            "key": self.key,
            "replaced": self.replaced,
        })
    }
}

/// One optimization step: a candidate, the gate decision, and why.
#[derive(Debug, Clone)]
pub struct Step {
    pub epoch: usize,
    pub batch: usize,
    pub phase: &'static str,
    pub proposed: Vec<Edit>,
    pub base_score: f64,
    pub candidate_score: f64,
    pub accepted: bool,
    pub reason: String,
}

impl Step {
    fn to_value(&self) -> Value {
        json!({
            "epoch": self.epoch,
            "batch": self.batch,
            "phase": self.phase,
            "edits": self.proposed.iter().map(Edit::to_value).collect::<Vec<_>>(),
            "base_score": self.base_score,
            "candidate_score": self.candidate_score,
            "accepted": self.accepted,
            "reason": self.reason,
        })
    }
}

/// Final outcome of an optimization run.
#[derive(Debug, Clone)]
pub struct TrainResult {
    pub initial_score: f64,
    pub final_score: f64,
    pub final_active: Vec<String>,
    pub rejected_buffer: Vec<String>,
    pub steps: Vec<Step>,
    pub adds: usize,
    pub deletes: usize,
    pub replaces: usize,
}

impl TrainResult {
    pub fn accepted_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.accepted).count()
    }

    pub fn rejected_steps(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !s.accepted && !s.proposed.is_empty())
            .count()
    }

    pub fn to_value(&self) -> Value {
        json!({
            "initial_score": self.initial_score,
            "final_score": self.final_score,
            "improvement": self.final_score - self.initial_score,
            "final_active": self.final_active,
            "rejected_buffer": self.rejected_buffer,
            "edits": { "add": self.adds, "delete": self.deletes, "replace": self.replaces },
            "accepted_steps": self.accepted_steps(),
            "rejected_steps": self.rejected_steps(),
            "steps": self.steps.iter().map(Step::to_value).collect::<Vec<_>>(),
        })
    }
}

/// The set of procedure keys "covered" by the active skill. An active key
/// covers itself and (one level) any key it supersedes, so a `replace` keeps
/// validation tasks that required the old procedure satisfied.
fn covered(active: &[String], catalog: &Catalog) -> HashSet<String> {
    let mut set = HashSet::new();
    for k in active {
        set.insert(k.clone());
        if let Some(old) = catalog.supersedes(k) {
            set.insert(old.clone());
        }
    }
    set
}

/// Held-out validation score: validation-task pass rate plus a small partial
/// required-key coverage term. Higher is better.
pub fn validation_score(tasks: &[Task], active: &[String], catalog: &Catalog) -> f64 {
    let val: Vec<&Task> = tasks.iter().filter(|t| t.split == Split::Val).collect();
    if val.is_empty() {
        return 0.0;
    }
    let cov = covered(active, catalog);
    let mut passed = 0usize;
    let mut req_total = 0usize;
    let mut req_hit = 0usize;
    for t in &val {
        let all = t.requires.iter().all(|r| cov.contains(r));
        if all {
            passed += 1;
        }
        for r in &t.requires {
            req_total += 1;
            if cov.contains(r) {
                req_hit += 1;
            }
        }
    }
    let pass_rate = passed as f64 / val.len() as f64;
    let coverage = if req_total == 0 {
        1.0
    } else {
        req_hit as f64 / req_total as f64
    };
    pass_rate + COVERAGE_WEIGHT * coverage
}

/// Reflect on a failure minibatch: rank the procedure keys missing from the
/// skill by how often they recur across failing rollouts, skipping anything in
/// the rejected buffer, and emit up to `budget` add/replace edits.
fn propose_edits(
    failures: &[&Task],
    active: &[String],
    catalog: &Catalog,
    rejected: &HashSet<String>,
    budget: usize,
) -> Vec<Edit> {
    let cov = covered(active, catalog);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for t in failures {
        for r in &t.requires {
            if !cov.contains(r) && !rejected.contains(r) {
                *counts.entry(r.clone()).or_insert(0) += 1;
            }
        }
    }
    // Deterministic ranking: recurrence desc, then key asc.
    let mut ranked: Vec<(String, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(budget);

    ranked
        .into_iter()
        .map(|(key, _)| match catalog.supersedes(&key) {
            Some(old) if active.iter().any(|a| a == old) => Edit {
                kind: EditKind::Replace,
                key,
                replaced: Some(old.clone()),
            },
            _ => Edit {
                kind: EditKind::Add,
                key,
                replaced: None,
            },
        })
        .collect()
}

/// Apply a set of edits to a copy of the active list, returning the candidate.
fn apply_edits(active: &[String], edits: &[Edit]) -> Vec<String> {
    let mut next = active.to_vec();
    for e in edits {
        match e.kind {
            EditKind::Add => {
                if !next.iter().any(|k| k == &e.key) {
                    next.push(e.key.clone());
                }
            }
            EditKind::Replace => {
                if let Some(old) = &e.replaced {
                    next.retain(|k| k != old);
                }
                if !next.iter().any(|k| k == &e.key) {
                    next.push(e.key.clone());
                }
            }
            EditKind::Delete => {
                next.retain(|k| k != &e.key);
            }
        }
    }
    next
}

/// Keys present in the skill but required by no rollout in any split — the
/// candidates the slow/meta update prunes.
fn stale_keys(active: &[String], tasks: &[Task]) -> Vec<String> {
    let mut required: HashSet<&str> = HashSet::new();
    for t in tasks {
        for r in &t.requires {
            required.insert(r.as_str());
        }
    }
    active
        .iter()
        .filter(|k| !required.contains(k.as_str()))
        .cloned()
        .collect()
}

/// Run the SkillOpt loop: rollout (offline) -> reflect -> bounded edit -> gate,
/// per minibatch and epoch, with a rejected-edit buffer and an epoch-wise slow
/// update that prunes stale procedures without regressing validation.
pub fn optimize(dataset: &Dataset, cfg: &OptConfig) -> TrainResult {
    let catalog = &dataset.catalog;
    let mut active = dataset.skill.active.clone();
    let mut rejected: HashSet<String> = HashSet::new();
    let mut steps: Vec<Step> = Vec::new();
    let (mut adds, mut deletes, mut replaces) = (0usize, 0usize, 0usize);

    let initial_score = validation_score(&dataset.tasks, &active, catalog);
    let mut base_score = initial_score;

    let train: Vec<&Task> = dataset
        .tasks
        .iter()
        .filter(|t| t.split == Split::Train)
        .collect();
    let batch_size = if cfg.batch_size == 0 {
        train.len().max(1)
    } else {
        cfg.batch_size
    };

    for epoch in 0..cfg.epochs {
        for (batch_idx, chunk) in train.chunks(batch_size).enumerate() {
            let failures: Vec<&Task> = chunk.iter().copied().filter(|t| !t.success).collect();
            let proposed = propose_edits(&failures, &active, catalog, &rejected, cfg.edit_budget);
            if proposed.is_empty() {
                continue;
            }
            let candidate = apply_edits(&active, &proposed);
            let candidate_score = validation_score(&dataset.tasks, &candidate, catalog);
            let prev_base = base_score;
            // The gate: accept only on a strict improvement past the margin.
            let accepted = candidate_score > prev_base + cfg.gate_margin;
            let reason = if accepted {
                format!("val {prev_base:.4} -> {candidate_score:.4} (> margin)")
            } else {
                "no strict val improvement; edits buffered".to_string()
            };
            if accepted {
                for e in &proposed {
                    match e.kind {
                        EditKind::Add => adds += 1,
                        EditKind::Replace => replaces += 1,
                        EditKind::Delete => deletes += 1,
                    }
                }
                active = candidate;
                base_score = candidate_score;
            } else {
                for e in &proposed {
                    rejected.insert(e.key.clone());
                }
            }
            steps.push(Step {
                epoch,
                batch: batch_idx,
                phase: "reflect",
                proposed,
                base_score: prev_base,
                candidate_score,
                accepted,
                reason,
            });
        }

        // Slow/meta update: prune stale procedures, gated so val never drops.
        let stale = stale_keys(&active, &dataset.tasks);
        for key in stale.into_iter().take(cfg.slow_cap) {
            let edit = Edit {
                kind: EditKind::Delete,
                key: key.clone(),
                replaced: None,
            };
            let candidate = apply_edits(&active, std::slice::from_ref(&edit));
            let candidate_score = validation_score(&dataset.tasks, &candidate, catalog);
            let prev_base = base_score;
            let accepted = candidate_score >= prev_base;
            let reason = if accepted {
                format!("pruned stale '{key}' without val regression")
            } else {
                format!("keep '{key}': pruning would drop val")
            };
            if accepted {
                deletes += 1;
                active = candidate;
                base_score = candidate_score;
            }
            steps.push(Step {
                epoch,
                batch: 0,
                phase: "slow-update",
                proposed: vec![edit],
                base_score: prev_base,
                candidate_score,
                accepted,
                reason,
            });
        }
    }

    let mut rejected_buffer: Vec<String> = rejected.into_iter().collect();
    rejected_buffer.sort();

    TrainResult {
        initial_score,
        final_score: base_score,
        final_active: active,
        rejected_buffer,
        steps,
        adds,
        deletes,
        replaces,
    }
}

/// Render the optimized skill as a single deployable markdown document.
pub fn render_skill(skill: &Skill, active: &[String], catalog: &Catalog) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n", skill.title));
    for section in &skill.sections {
        out.push('\n');
        if let Some(h) = &section.heading {
            out.push_str(&format!("## {h}\n\n"));
        }
        out.push_str(section.text.trim_end());
        out.push('\n');
    }
    out.push_str("\n## Procedimentos aprendidos (SkillOpt)\n\n");
    if active.is_empty() {
        out.push_str("_nenhum procedimento aprendido_\n");
    } else {
        for key in active {
            out.push_str(&format!("- {}\n", catalog.text_for(key)));
        }
    }
    out
}

/// Human-readable run summary (text mode), mirroring the other `lpm` reports.
pub fn explain(dataset: &Dataset, cfg: &OptConfig, result: &TrainResult) -> String {
    let batch = if cfg.batch_size == 0 {
        "all".to_string()
    } else {
        cfg.batch_size.to_string()
    };
    let mut out = String::new();
    out.push_str("SkillOpt — otimização de skill para modelo congelado\n");
    out.push_str(&format!(
        "epochs={} batch_size={} edit_budget={} gate_margin={:.4} slow_cap={}\n",
        cfg.epochs, batch, cfg.edit_budget, cfg.gate_margin, cfg.slow_cap
    ));
    let n_val = dataset
        .tasks
        .iter()
        .filter(|t| t.split == Split::Val)
        .count();
    let n_train = dataset.tasks.len() - n_val;
    out.push_str(&format!("rollouts: {n_train} train, {n_val} val\n"));
    out.push_str(&format!(
        "val inicial: {:.4}\nval final:   {:.4}  ({:+.4})\n",
        result.initial_score,
        result.final_score,
        result.final_score - result.initial_score
    ));
    out.push_str(&format!(
        "steps: {} (aceitos {}, rejeitados {}) | edits: add {}, replace {}, delete {}\n",
        result.steps.len(),
        result.accepted_steps(),
        result.rejected_steps(),
        result.adds,
        result.replaces,
        result.deletes,
    ));
    if !result.rejected_buffer.is_empty() {
        out.push_str(&format!(
            "buffer de rejeição: {}\n",
            result.rejected_buffer.join(", ")
        ));
    }
    out.push_str(&format!(
        "procedimentos finais ({}): {}\n",
        result.final_active.len(),
        if result.final_active.is_empty() {
            "—".to_string()
        } else {
            result.final_active.join(", ")
        }
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset(json_str: &str) -> Dataset {
        Dataset::from_value(&serde_json::from_str(json_str).unwrap())
    }

    const BASIC: &str = r#"{
        "skill": { "title": "Agente", "lessons": [] },
        "catalog": {
            "abs-paths": "Use caminhos absolutos ao editar arquivos.",
            "verify": "Rode o comando de verificação antes de concluir."
        },
        "tasks": [
            { "id": "tr1", "split": "train", "success": false, "requires": ["abs-paths"] },
            { "id": "tr2", "split": "train", "success": false, "requires": ["abs-paths", "verify"] },
            { "id": "v1",  "split": "val",   "success": false, "requires": ["abs-paths"] },
            { "id": "v2",  "split": "val",   "success": false, "requires": ["verify"] }
        ]
    }"#;

    #[test]
    fn parses_splits_and_outcomes() {
        let d = dataset(BASIC);
        assert_eq!(d.tasks.len(), 4);
        let val = d.tasks.iter().filter(|t| t.split == Split::Val).count();
        assert_eq!(val, 2);
        assert!(!d.tasks[0].success);
    }

    #[test]
    fn score_derived_from_numeric_reward() {
        let d = dataset(
            r#"{ "tasks": [
                { "split": "train", "score": 0.9, "requires": [] },
                { "split": "train", "score": 0.1, "requires": [] }
            ] }"#,
        );
        assert!(d.tasks[0].success);
        assert!(!d.tasks[1].success);
    }

    #[test]
    fn empty_skill_scores_zero_then_optimizes_to_full() {
        let d = dataset(BASIC);
        let cfg = OptConfig::default();
        let r = optimize(&d, &cfg);
        // Nothing learned at start: no val task can pass.
        assert!(r.initial_score < COVERAGE_WEIGHT);
        // Both procedures get added; both val tasks then pass.
        assert!((r.final_score - (1.0 + COVERAGE_WEIGHT)).abs() < 1e-9);
        assert!(r.final_active.contains(&"abs-paths".to_string()));
        assert!(r.final_active.contains(&"verify".to_string()));
        assert!(r.adds >= 2);
    }

    #[test]
    fn gate_rejects_edits_that_do_not_help_validation() {
        // The failing train task requires a procedure no val task needs:
        // adding it cannot improve the val score, so the gate must reject it
        // and the key must land in the rejected buffer.
        let d = dataset(
            r#"{
                "skill": { "title": "S", "lessons": [] },
                "tasks": [
                    { "split": "train", "success": false, "requires": ["irrelevant"] },
                    { "split": "val",   "success": true,  "requires": [] }
                ]
            }"#,
        );
        let r = optimize(&d, &OptConfig::default());
        assert!(r.rejected_buffer.contains(&"irrelevant".to_string()));
        assert!(!r.final_active.contains(&"irrelevant".to_string()));
        assert_eq!(r.adds, 0);
    }

    #[test]
    fn edit_budget_limits_edits_per_candidate() {
        let d = dataset(
            r#"{
                "skill": { "title": "S", "lessons": [] },
                "tasks": [
                    { "split": "train", "success": false, "requires": ["a","b","c","d"] },
                    { "split": "val",   "success": false, "requires": ["a","b","c","d"] }
                ]
            }"#,
        );
        let cfg = OptConfig {
            edit_budget: 1,
            ..OptConfig::default()
        };
        let r = optimize(&d, &cfg);
        // First reflect step may carry at most one edit (the learning rate).
        assert!(r.steps[0].proposed.len() <= 1);
    }

    #[test]
    fn replace_supersedes_old_procedure() {
        let d = dataset(
            r#"{
                "skill": { "title": "S", "lessons": ["old-rule"] },
                "catalog": {
                    "new-rule": { "text": "Regra nova.", "supersedes": "old-rule" }
                },
                "tasks": [
                    { "split": "train", "success": false, "requires": ["new-rule"] },
                    { "split": "val",   "success": false, "requires": ["new-rule"] }
                ]
            }"#,
        );
        let r = optimize(&d, &OptConfig::default());
        assert!(r.replaces >= 1);
        assert!(r.final_active.contains(&"new-rule".to_string()));
        assert!(!r.final_active.contains(&"old-rule".to_string()));
    }

    #[test]
    fn slow_update_prunes_stale_procedure() {
        // "bloat" is in the skill but required by no rollout -> pruned.
        let d = dataset(
            r#"{
                "skill": { "title": "S", "lessons": ["bloat", "keep"] },
                "tasks": [
                    { "split": "val", "success": true, "requires": ["keep"] }
                ]
            }"#,
        );
        let r = optimize(&d, &OptConfig::default());
        assert!(!r.final_active.contains(&"bloat".to_string()));
        assert!(r.final_active.contains(&"keep".to_string()));
        assert!(r.deletes >= 1);
    }

    #[test]
    fn render_includes_learned_bullets() {
        let d = dataset(BASIC);
        let r = optimize(&d, &OptConfig::default());
        let md = render_skill(&d.skill, &r.final_active, &d.catalog);
        assert!(md.contains("# Agente"));
        assert!(md.contains("Procedimentos aprendidos (SkillOpt)"));
        assert!(md.contains("Use caminhos absolutos"));
    }

    #[test]
    fn run_is_deterministic() {
        let d = dataset(BASIC);
        let a = optimize(&d, &OptConfig::default());
        let b = optimize(&d, &OptConfig::default());
        assert_eq!(a.final_active, b.final_active);
        assert_eq!(a.steps.len(), b.steps.len());
        assert!((a.final_score - b.final_score).abs() < 1e-12);
    }
}
