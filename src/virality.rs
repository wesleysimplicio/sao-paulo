//! Native (Rust) port of the `x-virality` scoring logic from
//! `x-virality-skills` (`src/x_virality_tools/score_simulator.py`).
//!
//! A heuristic weighted-score simulator for X (For You) posts: it combines
//! Phoenix per-candidate engagement predictions with the algorithm's scoring
//! weights, applies the negative-score offset, author-diversity decay, and the
//! out-of-network factor — the same pipeline the skill documents, so the LLM
//! can reason about what makes a post likely to rank.

use std::cmp::Ordering;

use serde_json::{json, Value};

pub const NEGATIVE_SCORES_OFFSET: f64 = 1.0;
pub const DEFAULT_MIN_VIDEO_DURATION_MS: i64 = 5_000;

#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub favorite: f64,
    pub reply: f64,
    pub retweet: f64,
    pub photo_expand: f64,
    pub click: f64,
    pub profile_click: f64,
    pub vqv: f64,
    pub share: f64,
    pub share_via_dm: f64,
    pub share_via_copy_link: f64,
    pub dwell: f64,
    pub quote: f64,
    pub quoted_click: f64,
    pub quoted_vqv: f64,
    pub cont_dwell_time: f64,
    pub cont_click_dwell_time: f64,
    pub follow_author: f64,
    pub not_interested: f64,
    pub block_author: f64,
    pub mute_author: f64,
    pub report: f64,
    pub not_dwelled: f64,
    pub min_video_duration_ms: i64,
    pub enable_quoted_vqv_duration_check: bool,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        ScoringWeights {
            favorite: 0.5,
            reply: 13.5,
            retweet: 1.0,
            photo_expand: 0.1,
            click: 0.1,
            profile_click: 12.0,
            vqv: 0.005,
            share: 1.0,
            share_via_dm: 1.0,
            share_via_copy_link: 1.0,
            dwell: 0.5,
            quote: 1.0,
            quoted_click: 1.0,
            quoted_vqv: 0.005,
            cont_dwell_time: 0.0001,
            cont_click_dwell_time: 0.0001,
            follow_author: 24.0,
            not_interested: -8.0,
            block_author: -80.0,
            mute_author: -40.0,
            report: -100.0,
            not_dwelled: -0.5,
            min_video_duration_ms: DEFAULT_MIN_VIDEO_DURATION_MS,
            enable_quoted_vqv_duration_check: true,
        }
    }
}

impl ScoringWeights {
    pub fn positive_sum(&self) -> f64 {
        self.favorite
            + self.reply
            + self.retweet
            + self.photo_expand
            + self.click
            + self.profile_click
            + self.vqv
            + self.share
            + self.share_via_dm
            + self.share_via_copy_link
            + self.dwell
            + self.quote
            + self.quoted_click
            + self.quoted_vqv
            + self.follow_author
    }

    pub fn negative_sum(&self) -> f64 {
        -(self.not_interested
            + self.block_author
            + self.mute_author
            + self.report
            + self.not_dwelled)
    }

    pub fn total_sum(&self) -> f64 {
        self.positive_sum() + self.negative_sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhoenixScores {
    pub favorite: f64,
    pub reply: f64,
    pub retweet: f64,
    pub photo_expand: f64,
    pub click: f64,
    pub profile_click: f64,
    pub vqv: f64,
    pub share: f64,
    pub share_via_dm: f64,
    pub share_via_copy_link: f64,
    pub dwell: f64,
    pub quote: f64,
    pub quoted_click: f64,
    pub quoted_vqv: f64,
    pub dwell_time: f64,
    pub click_dwell_time: f64,
    pub follow_author: f64,
    pub not_interested: f64,
    pub block_author: f64,
    pub mute_author: f64,
    pub report: f64,
    pub not_dwelled: f64,
}

impl PhoenixScores {
    fn from_value(v: &Value) -> Self {
        let g = |key: &str| v.get(key).and_then(Value::as_f64).unwrap_or(0.0);
        PhoenixScores {
            favorite: g("favorite"),
            reply: g("reply"),
            retweet: g("retweet"),
            photo_expand: g("photo_expand"),
            click: g("click"),
            profile_click: g("profile_click"),
            vqv: g("vqv"),
            share: g("share"),
            share_via_dm: g("share_via_dm"),
            share_via_copy_link: g("share_via_copy_link"),
            dwell: g("dwell"),
            quote: g("quote"),
            quoted_click: g("quoted_click"),
            quoted_vqv: g("quoted_vqv"),
            dwell_time: g("dwell_time"),
            click_dwell_time: g("click_dwell_time"),
            follow_author: g("follow_author"),
            not_interested: g("not_interested"),
            block_author: g("block_author"),
            mute_author: g("mute_author"),
            report: g("report"),
            not_dwelled: g("not_dwelled"),
        }
    }

    fn to_value(&self) -> Value {
        json!({
            "favorite": self.favorite,
            "reply": self.reply,
            "retweet": self.retweet,
            "photo_expand": self.photo_expand,
            "click": self.click,
            "profile_click": self.profile_click,
            "vqv": self.vqv,
            "share": self.share,
            "share_via_dm": self.share_via_dm,
            "share_via_copy_link": self.share_via_copy_link,
            "dwell": self.dwell,
            "quote": self.quote,
            "quoted_click": self.quoted_click,
            "quoted_vqv": self.quoted_vqv,
            "dwell_time": self.dwell_time,
            "click_dwell_time": self.click_dwell_time,
            "follow_author": self.follow_author,
            "not_interested": self.not_interested,
            "block_author": self.block_author,
            "mute_author": self.mute_author,
            "report": self.report,
            "not_dwelled": self.not_dwelled,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub scores: PhoenixScores,
    pub in_network: bool,
    pub video_duration_ms: Option<i64>,
    pub quoted_video_duration_ms: Option<i64>,
    pub author_id: i64,
}

impl Candidate {
    pub fn from_value(v: &Value) -> Self {
        Candidate {
            scores: PhoenixScores::from_value(v.get("scores").unwrap_or(&Value::Null)),
            in_network: v.get("in_network").and_then(Value::as_bool).unwrap_or(true),
            video_duration_ms: v.get("video_duration_ms").and_then(Value::as_i64),
            quoted_video_duration_ms: v.get("quoted_video_duration_ms").and_then(Value::as_i64),
            author_id: v.get("author_id").and_then(Value::as_i64).unwrap_or(0),
        }
    }
}

pub fn vqv_weight(candidate: &Candidate, weights: &ScoringWeights) -> f64 {
    match candidate.video_duration_ms {
        Some(ms) if ms > weights.min_video_duration_ms => weights.vqv,
        _ => 0.0,
    }
}

pub fn quoted_vqv_weight(candidate: &Candidate, weights: &ScoringWeights) -> f64 {
    if !weights.enable_quoted_vqv_duration_check {
        return weights.quoted_vqv;
    }
    match candidate.quoted_video_duration_ms {
        Some(ms) if ms > weights.min_video_duration_ms => weights.quoted_vqv,
        _ => 0.0,
    }
}

pub fn compute_weighted_score(candidate: &Candidate, weights: &ScoringWeights) -> f64 {
    let s = &candidate.scores;
    s.favorite * weights.favorite
        + s.reply * weights.reply
        + s.retweet * weights.retweet
        + s.photo_expand * weights.photo_expand
        + s.click * weights.click
        + s.profile_click * weights.profile_click
        + s.vqv * vqv_weight(candidate, weights)
        + s.share * weights.share
        + s.share_via_dm * weights.share_via_dm
        + s.share_via_copy_link * weights.share_via_copy_link
        + s.dwell * weights.dwell
        + s.quote * weights.quote
        + s.quoted_click * weights.quoted_click
        + s.quoted_vqv * quoted_vqv_weight(candidate, weights)
        + s.dwell_time * weights.cont_dwell_time
        + s.click_dwell_time * weights.cont_click_dwell_time
        + s.follow_author * weights.follow_author
        + s.not_interested * weights.not_interested
        + s.block_author * weights.block_author
        + s.mute_author * weights.mute_author
        + s.report * weights.report
        + s.not_dwelled * weights.not_dwelled
}

pub fn offset_score(combined: f64, weights: &ScoringWeights, negative_offset: f64) -> f64 {
    if weights.total_sum() == 0.0 {
        return combined.max(0.0);
    }
    if combined < 0.0 {
        return (combined + weights.negative_sum()) / weights.total_sum() * negative_offset;
    }
    combined + negative_offset
}

pub fn diversity_multiplier(position: i32, decay: f64, floor: f64) -> f64 {
    (1.0 - floor) * decay.powi(position) + floor
}

pub fn apply_author_diversity(
    candidates: &[Candidate],
    weighted: &[f64],
    decay: f64,
    floor: f64,
) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = weighted.iter().copied().enumerate().collect();
    // Stable sort by weight descending (ties keep ascending original index).
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let mut author_counts: std::collections::HashMap<i64, i32> = std::collections::HashMap::new();
    let mut adjusted = vec![0.0; candidates.len()];
    for (original_index, weight) in indexed {
        let author_id = candidates[original_index].author_id;
        let position = *author_counts.get(&author_id).unwrap_or(&0);
        author_counts.insert(author_id, position + 1);
        adjusted[original_index] = weight * diversity_multiplier(position, decay, floor);
    }
    adjusted
}

pub fn apply_oon(candidates: &[Candidate], adjusted: &[f64], oon_factor: f64) -> Vec<f64> {
    candidates
        .iter()
        .zip(adjusted.iter())
        .map(|(c, &score)| {
            if c.in_network {
                score
            } else {
                score * oon_factor
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ScoreReport {
    pub combined: f64,
    pub offset: f64,
    pub diversity_adjusted: f64,
    pub final_score: f64,
    pub vqv_eligible: bool,
    pub quoted_vqv_eligible: bool,
    pub in_network: bool,
    pub video_duration_ms: Option<i64>,
    pub quoted_video_duration_ms: Option<i64>,
    pub author_id: i64,
    pub scores: PhoenixScores,
}

impl ScoreReport {
    pub fn explain(&self) -> String {
        let vd = self
            .video_duration_ms
            .map(|v| v.to_string())
            .unwrap_or_else(|| "None".into());
        format!(
            "in_network={}\nvideo_duration_ms={} (vqv_eligible={})\ncombined_weighted_score={:.4}\noffset_score={:.4}\nafter_diversity={:.4}\nfinal={:.4}",
            bool_py(self.in_network),
            vd,
            bool_py(self.vqv_eligible),
            self.combined,
            self.offset,
            self.diversity_adjusted,
            self.final_score,
        )
    }

    pub fn to_value(&self) -> Value {
        json!({
            "candidate": {
                "scores": self.scores.to_value(),
                "in_network": self.in_network,
                "video_duration_ms": self.video_duration_ms,
                "quoted_video_duration_ms": self.quoted_video_duration_ms,
                "author_id": self.author_id,
            },
            "combined": self.combined,
            "offset": self.offset,
            "diversity_adjusted": self.diversity_adjusted,
            "final": self.final_score,
            "vqv_eligible": self.vqv_eligible,
            "quoted_vqv_eligible": self.quoted_vqv_eligible,
        })
    }
}

fn bool_py(v: bool) -> &'static str {
    if v {
        "True"
    } else {
        "False"
    }
}

pub struct ScoreOptions {
    pub diversity_decay: f64,
    pub diversity_floor: f64,
    pub oon_factor: f64,
}

impl Default for ScoreOptions {
    fn default() -> Self {
        ScoreOptions {
            diversity_decay: 0.7,
            diversity_floor: 0.3,
            oon_factor: 0.5,
        }
    }
}

pub fn score_batch(
    candidates: &[Candidate],
    weights: &ScoringWeights,
    opts: &ScoreOptions,
) -> Vec<ScoreReport> {
    let combined: Vec<f64> = candidates
        .iter()
        .map(|c| compute_weighted_score(c, weights))
        .collect();
    let offset: Vec<f64> = combined
        .iter()
        .map(|&c| offset_score(c, weights, NEGATIVE_SCORES_OFFSET))
        .collect();
    let diversity = apply_author_diversity(
        candidates,
        &offset,
        opts.diversity_decay,
        opts.diversity_floor,
    );
    let final_scores = apply_oon(candidates, &diversity, opts.oon_factor);

    candidates
        .iter()
        .enumerate()
        .map(|(i, c)| ScoreReport {
            combined: combined[i],
            offset: offset[i],
            diversity_adjusted: diversity[i],
            final_score: final_scores[i],
            vqv_eligible: vqv_weight(c, weights) > 0.0,
            quoted_vqv_eligible: quoted_vqv_weight(c, weights) > 0.0,
            in_network: c.in_network,
            video_duration_ms: c.video_duration_ms,
            quoted_video_duration_ms: c.quoted_video_duration_ms,
            author_id: c.author_id,
            scores: c.scores.clone(),
        })
        .collect()
}

/// Parse a JSON payload (object or array) into candidates.
pub fn candidates_from_payload(payload: &Value) -> Vec<Candidate> {
    match payload {
        Value::Array(items) => items.iter().map(Candidate::from_value).collect(),
        other => vec![Candidate::from_value(other)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(json_str: &str) -> Candidate {
        Candidate::from_value(&serde_json::from_str(json_str).unwrap())
    }

    #[test]
    fn reply_weight_dominates() {
        let w = ScoringWeights::default();
        let c = cand(r#"{"scores":{"reply":1.0}}"#);
        // One unit of predicted reply contributes the reply weight.
        assert!((compute_weighted_score(&c, &w) - 13.5).abs() < 1e-9);
    }

    #[test]
    fn vqv_requires_min_video_duration() {
        let w = ScoringWeights::default();
        let short = cand(r#"{"scores":{"vqv":1.0},"video_duration_ms":1000}"#);
        let long = cand(r#"{"scores":{"vqv":1.0},"video_duration_ms":6000}"#);
        assert_eq!(vqv_weight(&short, &w), 0.0);
        assert_eq!(vqv_weight(&long, &w), w.vqv);
        assert!(compute_weighted_score(&short, &w) == 0.0);
        assert!((compute_weighted_score(&long, &w) - w.vqv).abs() < 1e-12);
    }

    #[test]
    fn negative_signal_uses_offset_normalization() {
        let w = ScoringWeights::default();
        let c = cand(r#"{"scores":{"report":1.0}}"#);
        let combined = compute_weighted_score(&c, &w);
        assert!(combined < 0.0);
        let off = offset_score(combined, &w, NEGATIVE_SCORES_OFFSET);
        // Negative combined maps below the positive offset of 1.0.
        assert!(off < NEGATIVE_SCORES_OFFSET);
    }

    #[test]
    fn diversity_decays_repeated_author() {
        let m0 = diversity_multiplier(0, 0.7, 0.3);
        let m1 = diversity_multiplier(1, 0.7, 0.3);
        let m2 = diversity_multiplier(2, 0.7, 0.3);
        assert!((m0 - 1.0).abs() < 1e-12); // first slot unaffected
        assert!(m1 < m0 && m2 < m1); // later slots decay toward floor
        assert!(m2 > 0.3); // never below floor
    }

    #[test]
    fn oon_factor_penalizes_out_of_network() {
        let cands = vec![
            cand(r#"{"scores":{"favorite":1.0},"in_network":true,"author_id":1}"#),
            cand(r#"{"scores":{"favorite":1.0},"in_network":false,"author_id":2}"#),
        ];
        let w = ScoringWeights::default();
        let reports = score_batch(&cands, &w, &ScoreOptions::default());
        // Same raw signal; out-of-network candidate ends up lower.
        assert!(reports[1].final_score < reports[0].final_score);
    }

    #[test]
    fn batch_ranks_reply_over_favorite() {
        let cands = vec![
            cand(r#"{"scores":{"favorite":1.0},"author_id":1}"#),
            cand(r#"{"scores":{"reply":1.0},"author_id":2}"#),
        ];
        let w = ScoringWeights::default();
        let r = score_batch(&cands, &w, &ScoreOptions::default());
        assert!(r[1].final_score > r[0].final_score);
    }
}
