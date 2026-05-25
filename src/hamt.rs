//! Native (Rust) port of `scripts/build_hamt.py`.
//!
//! Builds a YOOL/HAMT agent catalog from AGENTS.md: parses `### Name` agent
//! blocks with `- key: value` fields (and nested `agent_terms`), hashes each
//! `yool_id` with BLAKE2b-64 truncated to 30 bits, inserts the leaves into a
//! Hash Array Mapped Trie (5 bits/level, 6 levels), and emits a catalog JSON
//! whose `id` is the sha256 of its canonical serialization. Output is
//! semantically identical to the Python builder (same hashes, slots, HAMT
//! structure); only the volatile `generated_at`/`source`/`id` differ per run.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::Blake2bVar;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

pub const BRANCH_BITS: u32 = 5;
pub const BRANCH_FACTOR: u32 = 1 << BRANCH_BITS; // 32
pub const MAX_LEVELS: u32 = 6;

fn blake2b8(name: &str) -> [u8; 8] {
    use blake2::digest::{Update, VariableOutput};
    let mut hasher = Blake2bVar::new(8).expect("blake2b-8");
    hasher.update(name.as_bytes());
    let mut out = [0u8; 8];
    hasher.finalize_variable(&mut out).expect("blake2b output");
    out
}

pub fn yool_hash(name: &str) -> u64 {
    let value = u64::from_be_bytes(blake2b8(name));
    value & ((1u64 << (BRANCH_BITS * MAX_LEVELS)) - 1)
}

pub fn hash_hex(name: &str) -> String {
    blake2b8(name).iter().map(|b| format!("{b:02x}")).collect()
}

pub fn slot_path(hash_value: u64) -> Vec<u32> {
    let mask = (BRANCH_FACTOR - 1) as u64;
    (0..MAX_LEVELS)
        .map(|level| ((hash_value >> ((MAX_LEVELS - 1 - level) * BRANCH_BITS)) & mask) as u32)
        .collect()
}

fn heading_anchor(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "agent".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_scalar(raw: &str) -> Value {
    let mut value = raw.trim();
    if value.len() >= 2 && value.starts_with('`') && value.ends_with('`') {
        value = &value[1..value.len() - 1];
    }
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Value::String(value[1..value.len() - 1].to_string());
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return Value::String(value[1..value.len() - 1].to_string());
    }
    let lower = value.to_lowercase();
    if lower == "true" || lower == "false" {
        return Value::Bool(lower == "true");
    }
    if is_int(value) {
        if let Ok(n) = value.parse::<i64>() {
            return json!(n);
        }
    }
    if is_float(value) {
        if let Ok(f) = value.parse::<f64>() {
            return json!(f);
        }
    }
    if value.len() >= 2 && value.starts_with('[') && value.ends_with(']') {
        let as_json = value.replace('\'', "\"");
        if let Ok(parsed) = serde_json::from_str::<Value>(&as_json) {
            return parsed;
        }
        let inner = &value[1..value.len() - 1];
        let parts: Vec<Value> = inner
            .split(',')
            .map(|item| item.trim().trim_matches(['`', '\'', '"']).to_string())
            .filter(|item| !item.is_empty())
            .map(Value::String)
            .collect();
        return Value::Array(parts);
    }
    Value::String(value.to_string())
}

fn is_int(s: &str) -> bool {
    let body = s.strip_prefix('-').unwrap_or(s);
    !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit())
}

fn is_float(s: &str) -> bool {
    let body = s.strip_prefix('-').unwrap_or(s);
    match body.split_once('.') {
        Some((a, b)) => {
            !a.is_empty()
                && !b.is_empty()
                && a.bytes().all(|c| c.is_ascii_digit())
                && b.bytes().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

fn is_word_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// Parse `key<spaces>:<spaces>value` where key is `[A-Za-z_][\w-]*`.
fn parse_key_value(s: &str) -> Option<(String, String)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !is_word_start(bytes[0]) {
        return None;
    }
    let mut i = 1;
    while i < bytes.len() && is_word_char(bytes[i]) {
        i += 1;
    }
    let key = &s[..i];
    let rest = s[i..].trim_start();
    let rest = rest.strip_prefix(':')?;
    Some((key.to_string(), rest.trim_start().to_string()))
}

/// Top-level field: `- key: value`.
fn match_field(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix('-')?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    parse_key_value(rest.trim_start())
}

/// Indented field: 2+ leading spaces then `key: value`.
fn match_indent_field(line: &str) -> Option<(String, String)> {
    let leading = line.len() - line.trim_start().len();
    if leading < 2 {
        return None;
    }
    parse_key_value(line.trim_start())
}

fn is_heading(line: &str) -> Option<String> {
    if line.starts_with("####") {
        return None;
    }
    let rest = line.strip_prefix("###")?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    Some(rest.trim().to_string())
}

fn parse_agent_terms(lines: &[&str], start: usize) -> (Map<String, Value>, usize) {
    let mut terms = Map::new();
    let mut index = start;
    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty() {
            index += 1;
            continue;
        }
        if line.starts_with("### ") || line.starts_with("- ") {
            break;
        }
        match match_indent_field(line) {
            Some((k, v)) => {
                terms.insert(k, parse_scalar(&v));
                index += 1;
            }
            None => break,
        }
    }
    (terms, index)
}

fn is_falsy(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64().map(|f| f == 0.0).unwrap_or(false),
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
    }
}

#[derive(Default)]
pub struct ParseResult {
    pub parsed: Vec<Map<String, Value>>,
    pub skipped: Vec<Value>,
}

pub fn parse_agents(markdown: &str, source_name: &str) -> ParseResult {
    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut result = ParseResult::default();
    let mut current_name: Option<String> = None;
    let mut current: Map<String, Value> = Map::new();
    let mut heading_line = 0usize;
    let mut index = 0usize;

    let required = ["yool_id", "authority", "lane", "agent_terms"];

    macro_rules! finalize {
        () => {
            if let Some(name) = current_name.take() {
                let fields = std::mem::take(&mut current);
                let missing: Vec<&str> = required
                    .iter()
                    .copied()
                    .filter(|f| fields.get(*f).map(is_falsy).unwrap_or(true))
                    .collect();
                if missing.is_empty() {
                    let mut entry = Map::new();
                    entry.insert("name".into(), Value::String(name.clone()));
                    entry.insert(
                        "source".into(),
                        json!({"file": source_name, "line": heading_line, "anchor": heading_anchor(&name)}),
                    );
                    for (k, v) in fields {
                        entry.insert(k, v);
                    }
                    result.parsed.push(entry);
                } else {
                    result.skipped.push(json!({"name": name, "missing": missing, "line": heading_line}));
                }
            }
        };
    }

    while index < lines.len() {
        let line = lines[index];
        if let Some(name) = is_heading(line) {
            finalize!();
            current_name = Some(name);
            heading_line = index + 1;
            index += 1;
            continue;
        }
        if current_name.is_none() {
            index += 1;
            continue;
        }
        if let Some((key, raw_value)) = match_field(line) {
            if key == "agent_terms" {
                let (terms, next) = parse_agent_terms(&lines, index + 1);
                current.insert(key, Value::Object(terms));
                index = next;
                continue;
            }
            current.insert(key, parse_scalar(&raw_value));
        }
        index += 1;
    }
    finalize!();
    result
}

// --- HAMT ---

enum Node {
    Leaf(Value),
    Branch {
        bitmap: u32,
        children: BTreeMap<u32, Node>,
    },
    Collision {
        hash_prefix: String,
        leaves: Vec<Value>,
    },
}

fn branch() -> Node {
    Node::Branch {
        bitmap: 0,
        children: BTreeMap::new(),
    }
}

fn insert_leaf(node: &mut Node, key: &str, value: &Value, hash_value: u64, level: u32) {
    let (bitmap, children) = match node {
        Node::Branch { bitmap, children } => (bitmap, children),
        _ => unreachable!("insert into non-branch"),
    };

    if level >= MAX_LEVELS {
        let slot = (hash_value & (BRANCH_FACTOR as u64 - 1)) as u32;
        match children.get_mut(&slot) {
            None => {
                *bitmap |= 1 << slot;
                children.insert(
                    slot,
                    Node::Collision {
                        hash_prefix: format!("{hash_value:08x}"),
                        leaves: vec![value.clone()],
                    },
                );
            }
            Some(Node::Collision { leaves, .. }) => leaves.push(value.clone()),
            Some(_) => panic!("unexpected non-collision node at max depth"),
        }
        return;
    }

    let slot = slot_path(hash_value)[level as usize];
    match children.get_mut(&slot) {
        None => {
            *bitmap |= 1 << slot;
            children.insert(slot, Node::Leaf(value.clone()));
        }
        Some(existing) => match existing {
            Node::Leaf(prior) => {
                let prior_yool = prior["yool_id"].as_str().unwrap_or_default().to_string();
                if prior_yool == key {
                    *existing = Node::Leaf(value.clone());
                } else {
                    let prior_value = prior.clone();
                    let mut sub = branch();
                    insert_leaf(
                        &mut sub,
                        &prior_yool,
                        &prior_value,
                        yool_hash(&prior_yool),
                        level + 1,
                    );
                    insert_leaf(&mut sub, key, value, hash_value, level + 1);
                    *existing = sub;
                }
            }
            Node::Branch { .. } => insert_leaf(existing, key, value, hash_value, level + 1),
            Node::Collision { leaves, .. } => leaves.push(value.clone()),
        },
    }
}

fn child_to_value(node: &Node) -> Value {
    match node {
        Node::Leaf(entry) => json!({"kind": "leaf", "entry": entry}),
        Node::Collision {
            hash_prefix,
            leaves,
        } => {
            json!({"kind": "collision", "hash_prefix": hash_prefix, "leaves": leaves})
        }
        Node::Branch { bitmap, children } => {
            let mut obj = Map::new();
            obj.insert("kind".into(), Value::String("node".into()));
            obj.insert("bitmap".into(), json!(bitmap));
            obj.insert("children".into(), children_to_value(children));
            Value::Object(obj)
        }
    }
}

fn children_to_value(children: &BTreeMap<u32, Node>) -> Value {
    let mut map = Map::new();
    for (slot, child) in children {
        map.insert(slot.to_string(), child_to_value(child));
    }
    Value::Object(map)
}

fn root_to_value(node: &Node) -> Value {
    if let Node::Branch { bitmap, children } = node {
        json!({"bitmap": bitmap, "children": children_to_value(children)})
    } else {
        unreachable!("root must be a branch")
    }
}

fn root_bitmap(node: &Node) -> u32 {
    match node {
        Node::Branch { bitmap, .. } => *bitmap,
        _ => 0,
    }
}

fn canonical_json(value: &Value) -> String {
    // serde_json Value uses a BTreeMap (sorted keys) by default and compact
    // serialization uses `,`/`:` separators with no spaces — matching Python's
    // json.dumps(sort_keys=True, separators=(",", ":"), ensure_ascii=False).
    serde_json::to_string(value).unwrap_or_default()
}

pub fn build_catalog(entries: &[Map<String, Value>], source: &str, generated_at: &str) -> Value {
    let mut root = branch();
    let mut leaves: Vec<Value> = Vec::new();

    for entry in entries {
        let yool_id = entry
            .get("yool_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let hash_value = yool_hash(&yool_id);
        let mut leaf = entry.clone();
        leaf.insert(
            "hash".into(),
            json!({
                "algorithm": "blake2b-64-truncated-30",
                "hex": hash_hex(&yool_id),
                "value": hash_value,
                "slots": slot_path(hash_value),
            }),
        );
        let leaf_value = Value::Object(leaf);
        leaves.push(leaf_value.clone());
        insert_leaf(&mut root, &yool_id, &leaf_value, hash_value, 0);
    }

    leaves.sort_by(|a, b| {
        a["yool_id"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["yool_id"].as_str().unwrap_or_default())
    });

    let mut base = Map::new();
    base.insert("schema".into(), Value::String("yool-catalog/v1".into()));
    base.insert(
        "generated_at".into(),
        Value::String(generated_at.to_string()),
    );
    base.insert("source".into(), Value::String(source.to_string()));
    base.insert("entries".into(), Value::Array(leaves.clone()));
    base.insert(
        "hamt".into(),
        json!({
            "algorithm": "hamt/blake2b-30/v1",
            "branch_bits": BRANCH_BITS,
            "branch_factor": BRANCH_FACTOR,
            "max_levels": MAX_LEVELS,
            "root": root_to_value(&root),
        }),
    );

    let base_value = Value::Object(base.clone());
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(&base_value).as_bytes());
    let id = format!("sha256:{:x}", hasher.finalize());

    base.insert("id".into(), Value::String(id));
    base.insert(
        "stats".into(),
        json!({
            "entries": leaves.len(),
            "root_popcount": root_bitmap(&root).count_ones(),
        }),
    );
    Value::Object(base)
}

/// UTC timestamp like Python's `datetime.now(timezone.utc).isoformat()`.
pub fn now_iso() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() as i64;
    let micros = dur.subsec_micros();
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (h, mi, s) = (tod / 3600, (tod % 3600) / 60, tod % 60);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{micros:06}+00:00")
}

fn civil_from_days(z0: i64) -> (i64, u32, u32) {
    let z = z0 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Build the catalog from an AGENTS.md path and write it to `output`.
/// Returns the printed summary lines, or an error message.
pub fn run(source: &Path, output: &Path) -> Result<Vec<String>, String> {
    let markdown = std::fs::read_to_string(source)
        .map_err(|_| format!("AGENTS source not found: {}", source.display()))?;
    let source_name = source
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let parsed = parse_agents(&markdown, &source_name);
    let mut catalog = build_catalog(&parsed.parsed, &source.display().to_string(), &now_iso());

    if let Some(stats) = catalog.get_mut("stats").and_then(Value::as_object_mut) {
        stats.insert("parsed_agents".into(), json!(parsed.parsed.len()));
        stats.insert("skipped_agents".into(), json!(parsed.skipped.len()));
    }
    let root_popcount = catalog["stats"]["root_popcount"].as_u64().unwrap_or(0);
    if !parsed.skipped.is_empty() {
        if let Some(obj) = catalog.as_object_mut() {
            obj.insert("skipped".into(), Value::Array(parsed.skipped.clone()));
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(&catalog).map_err(|e| e.to_string())? + "\n";
    std::fs::write(output, body).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    out.push(format!(
        "[build] parsed {} agent(s) from {}",
        parsed.parsed.len(),
        source_name
    ));
    if !parsed.skipped.is_empty() {
        out.push(format!(
            "[build] skipped {} incomplete agent(s)",
            parsed.skipped.len()
        ));
    }
    out.push(format!("[build] wrote {}", output.display()));
    out.push(format!(
        "[build] root popcount: {root_popcount}/{BRANCH_FACTOR}"
    ));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake2b_and_hash_match_python() {
        assert_eq!(hash_hex("agent.dev.python"), "f97ad90de3f202e9");
        assert_eq!(yool_hash("agent.dev.python"), 603_063_017);
        assert_eq!(
            slot_path(yool_hash("agent.dev.python")),
            vec![17, 31, 4, 0, 23, 9]
        );
        assert_eq!(hash_hex("agent.ops.rust"), "033df2454698ba66");
        assert_eq!(
            slot_path(yool_hash("agent.ops.rust")),
            vec![3, 9, 17, 14, 19, 6]
        );
        assert_eq!(hash_hex("kernel_root"), "b4d43feba222b684");
    }

    #[test]
    fn parse_scalar_variants() {
        assert_eq!(
            parse_scalar("`agent.dev.python`"),
            json!("agent.dev.python")
        );
        assert_eq!(parse_scalar("\"hi\""), json!("hi"));
        assert_eq!(parse_scalar("true"), json!(true));
        assert_eq!(parse_scalar("60"), json!(60));
        assert_eq!(parse_scalar("-3"), json!(-3));
        assert_eq!(parse_scalar("1.5"), json!(1.5));
        assert_eq!(parse_scalar("dev"), json!("dev"));
    }

    fn fixture() -> &'static str {
        "### Agent One\n\
         \n\
         - yool_id: `agent.dev.python`\n\
         - authority: dev\n\
         - lane: fast\n\
         - agent_terms:\n\
         \x20\x20\x20\x20cpu_quota_pct: 60\n\
         \x20\x20\x20\x20disk_quota_mb: 100\n\
         \x20\x20\x20\x20timeout_s: 300\n\
         \n\
         ### Agent Two\n\
         \n\
         - yool_id: `agent.ops.rust`\n\
         - authority: ops\n\
         - lane: slow\n\
         - agent_terms:\n\
         \x20\x20\x20\x20cpu_quota_pct: 40\n\
         \n\
         ### Incomplete\n\
         \n\
         - yool_id: `agent.x`\n"
    }

    #[test]
    fn parses_and_skips() {
        let r = parse_agents(fixture(), "AGENTS.md");
        assert_eq!(r.parsed.len(), 2);
        assert_eq!(r.skipped.len(), 1);
        assert_eq!(r.parsed[0]["yool_id"], json!("agent.dev.python"));
        assert_eq!(r.parsed[0]["agent_terms"]["cpu_quota_pct"], json!(60));
        assert_eq!(r.parsed[0]["source"]["anchor"], json!("agent-one"));
    }

    #[test]
    fn catalog_structure_and_stable_id() {
        let r = parse_agents(fixture(), "AGENTS.md");
        let a = build_catalog(&r.parsed, "AGENTS.md", "2026-01-01T00:00:00.000000+00:00");
        let b = build_catalog(&r.parsed, "AGENTS.md", "2026-01-01T00:00:00.000000+00:00");
        assert_eq!(a["id"], b["id"]); // deterministic for fixed inputs
        assert!(a["id"].as_str().unwrap().starts_with("sha256:"));
        assert_eq!(a["schema"], json!("yool-catalog/v1"));
        assert_eq!(a["stats"]["entries"], json!(2));
        // entries sorted by yool_id: agent.dev.python < agent.ops.rust
        assert_eq!(a["entries"][0]["yool_id"], json!("agent.dev.python"));
        assert_eq!(a["entries"][0]["hash"]["hex"], json!("f97ad90de3f202e9"));
        assert_eq!(a["hamt"]["branch_factor"], json!(32));
    }
}
