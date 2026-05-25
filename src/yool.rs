//! Native (Rust) port of the YOOL / tuple / HAMT kernel logic from
//! `simplicio-prompt` (`kernel/yool_tuple_kernel.py`).
//!
//! It implements the deterministic core of the capability-addressing pattern:
//! Linda-style tuple-space primitives (`out`/`in`/`rd`), Hilbert-indexed tuple
//! maps, content-addressable receipts, hierarchical `batch_spawn` that
//! represents 1,000,000+ virtual subagents without enumerating them,
//! `compress_token`/`prune_idle` for inactive materialized agents, env-driven
//! lane concurrency policy, and an auditable snapshot.
//!
//! The provider-facing runtime (async lane worker pool, circuit breaker,
//! jittered backoff) is intentionally out of scope here — this is the
//! deterministic logic that maps cleanly onto a native binary. Receipts use a
//! dependency-free FNV-1a content hash: stable content addresses, not a
//! cryptographic match to the Python blake2b digest.

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use serde_json::{json, Map, Value};

pub const DEFAULT_LANE_CONCURRENCY: i64 = 32;
pub const DEFAULT_MAX_LANE_CONCURRENCY: i64 = 64;
pub const DEFAULT_CPU_QUOTA_PCT: i64 = 95;
pub const DEFAULT_QUEUE_MAXSIZE: i64 = 8192;
pub const DEFAULT_COMPRESSION_THRESHOLD: i64 = 1024;
pub const DEFAULT_CACHE_MAX_ENTRIES: i64 = 16384;
pub const DEFAULT_CACHE_TTL_S: f64 = 3600.0;
pub const DEFAULT_API_MAX_RETRIES: i64 = 3;
pub const DEFAULT_API_BACKOFF_BASE_MS: i64 = 100;
pub const DEFAULT_API_BACKOFF_MAX_MS: i64 = 5000;
pub const DEFAULT_CIRCUIT_FAILURE_THRESHOLD: i64 = 5;
pub const DEFAULT_CIRCUIT_COOLDOWN_S: f64 = 30.0;
pub const DEFAULT_BATCH_SMALL_TASK_SIZE: i64 = 32;
pub const DEFAULT_CONTEXT_COMPRESSION_CHARS: i64 = 6000;

/// FNV-1a 64-bit content hash, hex-encoded — a stable content address.
pub fn content_hash(input: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn env_positive_int(names: &[&str], default: i64) -> i64 {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            let v = value.trim();
            if v.is_empty() {
                continue;
            }
            return match v.parse::<i64>() {
                Ok(parsed) if parsed > 0 => parsed,
                _ => default,
            };
        }
    }
    default
}

fn env_positive_float(names: &[&str], default: f64) -> f64 {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            let v = value.trim();
            if v.is_empty() {
                continue;
            }
            return match v.parse::<f64>() {
                Ok(parsed) if parsed > 0.0 => parsed,
                _ => default,
            };
        }
    }
    default
}

#[derive(Debug, Clone)]
pub struct RuntimePolicy {
    pub lane_concurrency: i64,
    pub max_lane_concurrency: i64,
    pub cpu_quota_pct: i64,
    pub queue_maxsize: i64,
    pub compression_threshold: i64,
    pub cache_max_entries: i64,
    pub cache_ttl_s: f64,
    pub api_max_retries: i64,
    pub api_backoff_base_ms: i64,
    pub api_backoff_max_ms: i64,
    pub circuit_failure_threshold: i64,
    pub circuit_cooldown_s: f64,
    pub batch_small_task_size: i64,
    pub context_compression_chars: i64,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        RuntimePolicy {
            lane_concurrency: DEFAULT_LANE_CONCURRENCY,
            max_lane_concurrency: DEFAULT_MAX_LANE_CONCURRENCY,
            cpu_quota_pct: DEFAULT_CPU_QUOTA_PCT,
            queue_maxsize: DEFAULT_QUEUE_MAXSIZE,
            compression_threshold: DEFAULT_COMPRESSION_THRESHOLD,
            cache_max_entries: DEFAULT_CACHE_MAX_ENTRIES,
            cache_ttl_s: DEFAULT_CACHE_TTL_S,
            api_max_retries: DEFAULT_API_MAX_RETRIES,
            api_backoff_base_ms: DEFAULT_API_BACKOFF_BASE_MS,
            api_backoff_max_ms: DEFAULT_API_BACKOFF_MAX_MS,
            circuit_failure_threshold: DEFAULT_CIRCUIT_FAILURE_THRESHOLD,
            circuit_cooldown_s: DEFAULT_CIRCUIT_COOLDOWN_S,
            batch_small_task_size: DEFAULT_BATCH_SMALL_TASK_SIZE,
            context_compression_chars: DEFAULT_CONTEXT_COMPRESSION_CHARS,
        }
    }
}

impl RuntimePolicy {
    pub fn from_env() -> Self {
        RuntimePolicy {
            lane_concurrency: env_positive_int(
                &["YOOL_TUPLE_LANE_CONCURRENCY", "YOOL_LANE_CONCURRENCY"],
                DEFAULT_LANE_CONCURRENCY,
            ),
            max_lane_concurrency: env_positive_int(
                &[
                    "YOOL_TUPLE_MAX_LANE_CONCURRENCY",
                    "YOOL_MAX_LANE_CONCURRENCY",
                ],
                DEFAULT_MAX_LANE_CONCURRENCY,
            )
            .max(1),
            cpu_quota_pct: env_positive_int(
                &["YOOL_TUPLE_CPU_QUOTA_PCT", "YOOL_CPU_QUOTA_PCT"],
                DEFAULT_CPU_QUOTA_PCT,
            )
            .clamp(1, 100),
            queue_maxsize: env_positive_int(
                &["YOOL_TUPLE_QUEUE_MAXSIZE", "YOOL_QUEUE_MAXSIZE"],
                DEFAULT_QUEUE_MAXSIZE,
            )
            .max(1),
            compression_threshold: env_positive_int(
                &[
                    "YOOL_TUPLE_COMPRESSION_THRESHOLD",
                    "YOOL_COMPRESSION_THRESHOLD",
                ],
                DEFAULT_COMPRESSION_THRESHOLD,
            )
            .max(1),
            cache_max_entries: env_positive_int(
                &["YOOL_TUPLE_CACHE_MAX_ENTRIES", "YOOL_CACHE_MAX_ENTRIES"],
                DEFAULT_CACHE_MAX_ENTRIES,
            )
            .max(1),
            cache_ttl_s: env_positive_float(
                &["YOOL_TUPLE_CACHE_TTL_S", "YOOL_CACHE_TTL_S"],
                DEFAULT_CACHE_TTL_S,
            )
            .max(0.001),
            api_max_retries: env_positive_int(
                &["YOOL_TUPLE_API_MAX_RETRIES", "YOOL_API_MAX_RETRIES"],
                DEFAULT_API_MAX_RETRIES,
            )
            .max(0),
            api_backoff_base_ms: env_positive_int(
                &["YOOL_TUPLE_API_BACKOFF_BASE_MS", "YOOL_API_BACKOFF_BASE_MS"],
                DEFAULT_API_BACKOFF_BASE_MS,
            )
            .max(1),
            api_backoff_max_ms: env_positive_int(
                &["YOOL_TUPLE_API_BACKOFF_MAX_MS", "YOOL_API_BACKOFF_MAX_MS"],
                DEFAULT_API_BACKOFF_MAX_MS,
            )
            .max(1),
            circuit_failure_threshold: env_positive_int(
                &[
                    "YOOL_TUPLE_CIRCUIT_FAILURE_THRESHOLD",
                    "YOOL_CIRCUIT_FAILURE_THRESHOLD",
                ],
                DEFAULT_CIRCUIT_FAILURE_THRESHOLD,
            )
            .max(1),
            circuit_cooldown_s: env_positive_float(
                &["YOOL_TUPLE_CIRCUIT_COOLDOWN_S", "YOOL_CIRCUIT_COOLDOWN_S"],
                DEFAULT_CIRCUIT_COOLDOWN_S,
            )
            .max(0.001),
            batch_small_task_size: env_positive_int(
                &[
                    "YOOL_TUPLE_BATCH_SMALL_TASK_SIZE",
                    "YOOL_BATCH_SMALL_TASK_SIZE",
                ],
                DEFAULT_BATCH_SMALL_TASK_SIZE,
            )
            .max(1),
            context_compression_chars: env_positive_int(
                &[
                    "YOOL_TUPLE_CONTEXT_COMPRESSION_CHARS",
                    "YOOL_CONTEXT_COMPRESSION_CHARS",
                ],
                DEFAULT_CONTEXT_COMPRESSION_CHARS,
            )
            .max(64),
        }
    }

    /// Bounded lane concurrency given queued roots and runtime signals.
    pub fn concurrency_for(
        &self,
        queued_roots: i64,
        ewma_latency_ms: Option<f64>,
        error_rate: f64,
    ) -> i64 {
        let mut requested = self.lane_concurrency;
        if requested <= 0 {
            let cpus = std::thread::available_parallelism()
                .map(|n| n.get() as i64)
                .unwrap_or(1);
            requested = queued_roots.max(1).min(cpus.max(1));
        }
        let ceiling = self.max_lane_concurrency.min(queued_roots.max(1)).max(1);
        let mut concurrency = requested.min(ceiling).max(1);
        if queued_roots > requested * 4 {
            concurrency = ceiling.min(concurrency.max(requested * 2));
        }
        if let Some(latency) = ewma_latency_ms {
            if latency > 250.0 {
                concurrency = ceiling.min(concurrency.max(concurrency * 2));
            }
        }
        if error_rate >= 0.2 {
            concurrency = (concurrency / 2).max(1);
        }
        concurrency.max(1)
    }
}

/// Simplified Hilbert index: multi-dimensional keys -> stable path (identity).
pub fn hilbert_compute(keys: &[i64]) -> Vec<i64> {
    keys.to_vec()
}

#[derive(Debug, Clone)]
pub struct YoolTuple {
    pub id: Option<u64>,
    pub yool: String,
    pub map: Vec<i64>,
    pub authority: String,
    pub lane: String,
    pub source: String,
    pub parent_id: Option<u64>,
    pub receipts: Vec<String>,
    pub data: Map<String, Value>,
    pub last_active: u64,
}

impl YoolTuple {
    pub fn new(yool: &str, map: Vec<i64>, authority: &str, lane: &str, source: &str) -> Self {
        YoolTuple {
            id: None,
            yool: yool.to_string(),
            map,
            authority: authority.to_string(),
            lane: lane.to_string(),
            source: source.to_string(),
            parent_id: None,
            receipts: Vec::new(),
            data: Map::new(),
            last_active: 0,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "yool": self.yool,
            "map": self.map,
            "authority": self.authority,
            "lane": self.lane,
            "source": self.source,
            "parent_id": self.parent_id,
            "receipts": self.receipts,
            "data": Value::Object(self.data.clone()),
        })
    }

    fn field_value(&self, key: &str) -> Option<Value> {
        match key {
            "id" => Some(self.id.map(|v| json!(v)).unwrap_or(Value::Null)),
            "yool" => Some(Value::String(self.yool.clone())),
            "authority" => Some(Value::String(self.authority.clone())),
            "lane" => Some(Value::String(self.lane.clone())),
            "source" => Some(Value::String(self.source.clone())),
            "parent_id" => Some(self.parent_id.map(|v| json!(v)).unwrap_or(Value::Null)),
            "map" => Some(json!(self.map)),
            other => self.data.get(other).cloned(),
        }
    }

    fn matches(&self, template: &Map<String, Value>) -> bool {
        for (key, expected) in template {
            match self.field_value(key) {
                Some(value) if &value == expected => {}
                _ => return false,
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct CompressToken {
    pub agent_id: u64,
    pub yool: String,
    pub map_index: Vec<i64>,
    pub authority: String,
    pub lane: String,
    pub source: String,
    pub parent_id: Option<u64>,
    pub receipts: Vec<String>,
    pub data_digest: String,
}

#[derive(Debug, Clone)]
pub struct BatchSpawnReceipt {
    pub root_agent_id: u64,
    pub depth: i64,
    pub branching: i64,
    pub virtual_agents: u128,
    pub compression_threshold: i64,
    pub receipt_id: String,
}

/// Small LRU+TTL cache keyed by content-addressable receipt/input hashes.
pub struct ReceiptCache {
    max_entries: usize,
    ttl_s: f64,
    items: Vec<(String, Value, Instant)>,
}

impl ReceiptCache {
    pub fn new(max_entries: usize, ttl_s: f64) -> Self {
        ReceiptCache {
            max_entries: max_entries.max(1),
            ttl_s: ttl_s.max(0.001),
            items: Vec::new(),
        }
    }

    pub fn get(&mut self, keys: &[String]) -> Option<(Value, String)> {
        let ttl = self.ttl_s;
        self.items
            .retain(|(_, _, created)| created.elapsed().as_secs_f64() <= ttl);
        for key in keys {
            if let Some(pos) = self.items.iter().position(|(k, _, _)| k == key) {
                let entry = self.items.remove(pos);
                let value = entry.1.clone();
                let k = entry.0.clone();
                self.items.push(entry);
                return Some((value, k));
            }
        }
        None
    }

    pub fn set(&mut self, keys: &[String], value: Value) {
        let now = Instant::now();
        for key in keys {
            self.items.retain(|(k, _, _)| k != key);
            self.items.push((key.clone(), value.clone(), now));
        }
        while self.items.len() > self.max_entries {
            self.items.remove(0);
        }
    }

    pub fn snapshot(&self) -> Value {
        json!({
            "entries": self.items.len(),
            "max_entries": self.max_entries,
            "ttl_s": self.ttl_s,
        })
    }
}

/// Tuple space with indexed lanes and lazy hierarchical (virtual) agents.
pub struct TupleSpace {
    pub policy: RuntimePolicy,
    store: HashMap<u64, YoolTuple>,
    space_index: HashMap<Vec<i64>, Vec<u64>>,
    lane_index: BTreeMap<String, Vec<u64>>,
    agents: BTreeMap<u64, u64>, // agent_id -> slot
    walls: BTreeMap<String, Vec<String>>,
    compressed_agents: BTreeMap<u64, CompressToken>,
    pub receipt_cache: ReceiptCache,
    virtual_agent_count: u128,
    next_agent_id: u64,
    next_slot: u64,
    tick: u64,
}

impl TupleSpace {
    pub fn new(policy: RuntimePolicy) -> Self {
        let cache = ReceiptCache::new(policy.cache_max_entries as usize, policy.cache_ttl_s);
        TupleSpace {
            policy,
            store: HashMap::new(),
            space_index: HashMap::new(),
            lane_index: BTreeMap::new(),
            agents: BTreeMap::new(),
            walls: BTreeMap::new(),
            compressed_agents: BTreeMap::new(),
            receipt_cache: cache,
            virtual_agent_count: 0,
            next_agent_id: 0,
            next_slot: 0,
            tick: 0,
        }
    }

    fn next_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    /// Insert a tuple into the space + lane index, append an `out@` receipt.
    pub fn out_tuple(&mut self, mut t: YoolTuple) -> u64 {
        let slot = self.next_slot;
        self.next_slot += 1;
        let receipt = format!(
            "out@{}",
            content_hash(&format!("{:?}|{}|{}", t.map, t.yool, t.receipts.len()))
        );
        t.receipts.push(receipt);
        t.last_active = self.next_tick();
        self.space_index
            .entry(t.map.clone())
            .or_default()
            .push(slot);
        self.lane_index
            .entry(t.lane.clone())
            .or_default()
            .push(slot);
        self.store.insert(slot, t);
        slot
    }

    fn allocate_agent_id(&mut self) -> u64 {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        id
    }

    /// Spawn one materialized subagent under `parent_slot`.
    pub fn spawn_agent(
        &mut self,
        parent_slot: u64,
        agent_yool: &str,
        agent_data: Map<String, Value>,
    ) -> u64 {
        let parent = self.store.get(&parent_slot).expect("parent slot exists");
        let parent_map = parent.map.clone();
        let parent_lane = parent.lane.clone();
        let parent_authority = parent.authority.clone();
        let parent_id = parent.id;

        let agent_id = self.allocate_agent_id();
        let mut keys = parent_map;
        keys.push(agent_id as i64);
        let map_idx = hilbert_compute(&keys);
        let lane = agent_data
            .get("lane")
            .and_then(Value::as_str)
            .unwrap_or(&parent_lane)
            .to_string();

        let mut tuple = YoolTuple::new(
            agent_yool,
            map_idx,
            &format!("subagent_{agent_id}"),
            &lane,
            &format!("spawned_from_{parent_authority}"),
        );
        tuple.id = Some(agent_id);
        tuple.parent_id = parent_id;
        tuple.data = agent_data;

        let slot = self.out_tuple(tuple);
        self.agents.insert(agent_id, slot);
        self.prune_idle(Some(self.policy.compression_threshold));
        agent_id
    }

    /// Lazy deep hierarchy: represents `branching^depth` virtual subagents while
    /// materializing only one controller tuple.
    pub fn batch_spawn(
        &mut self,
        parent_slot: u64,
        agent_yool: &str,
        depth: i64,
        branching: i64,
        compression_threshold: Option<i64>,
        agent_data: Option<Map<String, Value>>,
    ) -> Result<BatchSpawnReceipt, String> {
        if depth < 1 {
            return Err("depth must be >= 1".into());
        }
        if branching < 1 {
            return Err("branching must be >= 1".into());
        }
        let threshold = compression_threshold.unwrap_or(self.policy.compression_threshold);
        let virtual_agents = (branching as u128).saturating_pow(depth as u32);

        let mut data = agent_data.unwrap_or_default();
        data.insert("lazy_batch".into(), json!(true));
        data.insert("depth".into(), json!(depth));
        data.insert("branching".into(), json!(branching));
        data.insert(
            "virtual_agents".into(),
            virtual_agents
                .try_into()
                .map(|v: u64| json!(v))
                .unwrap_or_else(|_| json!(virtual_agents.to_string())),
        );
        data.insert("compression_threshold".into(), json!(threshold));

        let controller_id = self.spawn_agent(parent_slot, agent_yool, data);
        self.virtual_agent_count += virtual_agents;

        let receipt_id = content_hash(&format!(
            "({}, {}, {})|{}|{}",
            controller_id, depth, branching, agent_yool, virtual_agents
        ));
        if let Some(&slot) = self.agents.get(&controller_id) {
            if let Some(controller) = self.store.get_mut(&slot) {
                controller
                    .receipts
                    .push(format!("batch_spawn@{receipt_id}"));
            }
        }
        if self.agents.len() as i64 > threshold {
            self.compress_token(controller_id);
        }

        Ok(BatchSpawnReceipt {
            root_agent_id: controller_id,
            depth,
            branching,
            virtual_agents,
            compression_threshold: threshold,
            receipt_id,
        })
    }

    /// Compact a materialized agent into an auditable token and drop the tuple.
    pub fn compress_token(&mut self, agent_id: u64) -> Option<CompressToken> {
        let slot = match self.agents.get(&agent_id) {
            Some(&slot) => slot,
            None => return self.compressed_agents.get(&agent_id).cloned(),
        };
        let tup = self.store.get(&slot)?.clone();

        let mut sorted: Vec<(&String, &Value)> = tup.data.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        let data_digest = content_hash(&format!("{sorted:?}"));

        let token = CompressToken {
            agent_id,
            yool: tup.yool.clone(),
            map_index: tup.map.clone(),
            authority: tup.authority.clone(),
            lane: tup.lane.clone(),
            source: tup.source.clone(),
            parent_id: tup.parent_id,
            receipts: tup.receipts.clone(),
            data_digest,
        };

        self.remove_slot(slot, &tup.map, &tup.lane);
        self.agents.remove(&agent_id);
        self.compressed_agents.insert(agent_id, token.clone());
        Some(token)
    }

    /// Compress least-recently-active agents above `max_active`.
    pub fn prune_idle(&mut self, max_active: Option<i64>) -> usize {
        let max_active = max_active
            .unwrap_or(self.policy.compression_threshold)
            .max(0) as usize;
        if self.agents.len() <= max_active {
            return 0;
        }
        let mut active: Vec<(u64, u64)> = self
            .agents
            .iter()
            .filter_map(|(&agent_id, &slot)| {
                self.store.get(&slot).map(|t| (agent_id, t.last_active))
            })
            .collect();
        active.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        let to_compress = self.agents.len() - max_active;
        for (agent_id, _) in active.into_iter().take(to_compress) {
            self.compress_token(agent_id);
        }
        to_compress
    }

    fn remove_slot(&mut self, slot: u64, map: &[i64], lane: &str) {
        if let Some(list) = self.space_index.get_mut(map) {
            list.retain(|&s| s != slot);
            if list.is_empty() {
                self.space_index.remove(map);
            }
        }
        if let Some(list) = self.lane_index.get_mut(lane) {
            list.retain(|&s| s != slot);
            if list.is_empty() {
                self.lane_index.remove(lane);
            }
        }
        self.store.remove(&slot);
    }

    fn find_slot(&self, template: &Map<String, Value>) -> Option<u64> {
        let candidates: Vec<u64> = match template.get("lane").and_then(Value::as_str) {
            Some(lane) => self.lane_index.get(lane).cloned().unwrap_or_default(),
            None => self.store.keys().copied().collect(),
        };
        candidates.into_iter().find(|slot| {
            self.store
                .get(slot)
                .map(|t| t.matches(template))
                .unwrap_or(false)
        })
    }

    pub fn rd_tuple(&self, template: &Map<String, Value>) -> Option<YoolTuple> {
        self.find_slot(template)
            .and_then(|s| self.store.get(&s).cloned())
    }

    pub fn in_tuple(&mut self, template: &Map<String, Value>) -> Option<YoolTuple> {
        let slot = self.find_slot(template)?;
        let tup = self.store.get(&slot)?.clone();
        self.remove_slot(slot, &tup.map, &tup.lane);
        if let Some(id) = tup.id {
            self.agents.remove(&id);
        }
        Some(tup)
    }

    pub fn scan_index(
        &self,
        lane: Option<&str>,
        yool: Option<&str>,
        limit: usize,
    ) -> Vec<YoolTuple> {
        let slots: Vec<u64> = match lane {
            Some(l) => self.lane_index.get(l).cloned().unwrap_or_default(),
            None => {
                let mut all: Vec<u64> = self.store.keys().copied().collect();
                all.sort_unstable();
                all
            }
        };
        let mut out = Vec::new();
        for slot in slots {
            if let Some(t) = self.store.get(&slot) {
                if yool.is_none() || Some(t.yool.as_str()) == yool {
                    out.push(t.clone());
                }
                if out.len() >= limit {
                    break;
                }
            }
        }
        out
    }

    pub fn route_packet(&mut self, packet: &Map<String, Value>, target_lane: &str) -> bool {
        let mut template = Map::new();
        template.insert("lane".into(), Value::String(target_lane.to_string()));
        let slot = match self.find_slot(&template) {
            Some(s) => s,
            None => return false,
        };
        let tick = self.next_tick();
        if let Some(target) = self.store.get_mut(&slot) {
            for (k, v) in packet {
                target.data.insert(k.clone(), v.clone());
            }
            let receipt = format!(
                "route@{}",
                content_hash(&format!(
                    "{:?}|{}|{}",
                    target.map,
                    target_lane,
                    packet.len()
                ))
            );
            target.receipts.push(receipt);
            target.last_active = tick;
            true
        } else {
            false
        }
    }

    pub fn hookwall(&mut self, wall_id: &str, capability: &str, action: &str) -> bool {
        match action {
            "hook" => {
                let caps = self.walls.entry(wall_id.to_string()).or_default();
                if !caps.iter().any(|c| c == capability) {
                    caps.push(capability.to_string());
                }
                true
            }
            "check" => self
                .walls
                .get(wall_id)
                .map(|caps| caps.iter().any(|c| c == capability))
                .unwrap_or(false),
            "unhook" => {
                if let Some(caps) = self.walls.get_mut(wall_id) {
                    if let Some(pos) = caps.iter().position(|c| c == capability) {
                        caps.remove(pos);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn virtual_agent_count(&self) -> u128 {
        self.virtual_agent_count
    }

    pub fn active_agents(&self) -> usize {
        self.agents.len()
    }

    pub fn total_agents(&self) -> u128 {
        self.agents.len() as u128 + self.compressed_agents.len() as u128 + self.virtual_agent_count
    }

    pub fn snapshot(&self) -> Value {
        let tuples: usize = self.space_index.values().map(|v| v.len()).sum();
        let lanes: Map<String, Value> = self
            .lane_index
            .iter()
            .map(|(lane, items)| (lane.clone(), json!(items.len())))
            .collect();
        json!({
            "tuples": tuples,
            "lanes": Value::Object(lanes),
            "active_agents": self.agents.len(),
            "compressed_agents": self.compressed_agents.len(),
            "virtual_agents": num_value(self.virtual_agent_count),
            "total_agents": num_value(self.total_agents()),
            "policy": {
                "lane_concurrency": self.policy.lane_concurrency,
                "max_lane_concurrency": self.policy.max_lane_concurrency,
                "cpu_quota_pct": self.policy.cpu_quota_pct,
                "queue_maxsize": self.policy.queue_maxsize,
                "compression_threshold": self.policy.compression_threshold,
                "cache_max_entries": self.policy.cache_max_entries,
                "cache_ttl_s": self.policy.cache_ttl_s,
                "api_max_retries": self.policy.api_max_retries,
                "api_backoff_base_ms": self.policy.api_backoff_base_ms,
                "api_backoff_max_ms": self.policy.api_backoff_max_ms,
                "circuit_failure_threshold": self.policy.circuit_failure_threshold,
                "circuit_cooldown_s": self.policy.circuit_cooldown_s,
                "batch_small_task_size": self.policy.batch_small_task_size,
                "context_compression_chars": self.policy.context_compression_chars,
            },
            "cache": self.receipt_cache.snapshot(),
        })
    }
}

fn num_value(v: u128) -> Value {
    u64::try_from(v)
        .map(|n| json!(n))
        .unwrap_or_else(|_| json!(v.to_string()))
}

/// Default space + root tuple, matching the Python reference.
pub fn build_default_space() -> (TupleSpace, u64) {
    let mut ts = TupleSpace::new(RuntimePolicy::from_env());
    let root = YoolTuple::new("kernel_root", hilbert_compute(&[0]), "root", "main", "user");
    let root_slot = ts.out_tuple(root);
    (ts, root_slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn content_hash_is_stable_and_addressable() {
        assert_eq!(content_hash("abc"), content_hash("abc"));
        assert_ne!(content_hash("abc"), content_hash("abd"));
        assert_eq!(content_hash("abc").len(), 16);
    }

    #[test]
    fn hilbert_identity() {
        assert_eq!(hilbert_compute(&[0, 1, 2]), vec![0, 1, 2]);
    }

    #[test]
    fn spawn_extends_parent_map_and_indexes() {
        let (mut ts, root) = build_default_space();
        let a = ts.spawn_agent(root, "hamt_builder", data(&[("status", json!("ready"))]));
        assert_eq!(a, 0);
        let found = ts.scan_index(Some("main"), Some("hamt_builder"), 10);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].map, vec![0, 0]); // root map [0] + agent_id 0
        assert_eq!(found[0].parent_id, None);
    }

    #[test]
    fn batch_spawn_represents_a_million_virtual_agents() {
        let (mut ts, root) = build_default_space();
        ts.spawn_agent(root, "hamt_builder", data(&[("status", json!("ready"))]));
        let receipt = ts
            .batch_spawn(root, "codex_worker", 4, 32, Some(128), None)
            .unwrap();

        assert_eq!(receipt.virtual_agents, 1_048_576);
        assert_eq!(ts.virtual_agent_count(), 1_048_576);
        // Only two tuples were materialized for a million-subagent task.
        assert_eq!(ts.active_agents(), 2);
        assert_eq!(ts.total_agents(), 1_048_578);

        let snap = ts.snapshot();
        assert_eq!(snap["virtual_agents"], json!(1_048_576u64));
        assert_eq!(snap["total_agents"], json!(1_048_578u64));
        assert_eq!(snap["tuples"], json!(3)); // root + 2 controllers
        assert!(receipt.receipt_id.len() == 16);
    }

    #[test]
    fn batch_spawn_rejects_bad_args() {
        let (mut ts, root) = build_default_space();
        assert!(ts.batch_spawn(root, "w", 0, 2, None, None).is_err());
        assert!(ts.batch_spawn(root, "w", 2, 0, None, None).is_err());
    }

    #[test]
    fn prune_idle_compresses_least_active() {
        let mut ts = TupleSpace::new(RuntimePolicy::default());
        let root = ts.out_tuple(YoolTuple::new("root", vec![0], "root", "main", "user"));
        for i in 0..5 {
            ts.spawn_agent(root, "w", data(&[("n", json!(i))]));
        }
        assert_eq!(ts.active_agents(), 5);
        let compressed = ts.prune_idle(Some(2));
        assert_eq!(compressed, 3);
        assert_eq!(ts.active_agents(), 2);
    }

    #[test]
    fn hookwall_hook_check_unhook() {
        let mut ts = TupleSpace::new(RuntimePolicy::default());
        assert!(ts.hookwall("w", "cap", "hook"));
        assert!(ts.hookwall("w", "cap", "check"));
        assert!(ts.hookwall("w", "cap", "unhook"));
        assert!(!ts.hookwall("w", "cap", "check"));
    }

    #[test]
    fn route_packet_updates_lane_target() {
        let (mut ts, _root) = build_default_space();
        let packet = data(&[("payload", json!(42))]);
        assert!(ts.route_packet(&packet, "main"));
        assert!(!ts.route_packet(&packet, "missing-lane"));
    }

    #[test]
    fn concurrency_is_bounded() {
        let policy = RuntimePolicy::default();
        assert_eq!(policy.concurrency_for(1, None, 0.0), 1);
        let many = policy.concurrency_for(1000, None, 0.0);
        assert!(many <= policy.max_lane_concurrency);
        // High error rate halves concurrency.
        let calm = policy.concurrency_for(50, None, 0.0);
        let stressed = policy.concurrency_for(50, None, 0.5);
        assert!(stressed <= calm);
    }

    #[test]
    fn receipt_cache_roundtrip() {
        let mut cache = ReceiptCache::new(2, 60.0);
        cache.set(&["k1".into()], json!({"v": 1}));
        assert!(cache.get(&["k1".into()]).is_some());
        assert!(cache.get(&["missing".into()]).is_none());
    }
}
