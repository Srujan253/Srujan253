use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::collect::ActivitySnapshot;

pub const MAX_RED_APPLES_PER_DAY: u32 = 3;
pub const MAX_GOLDEN_APPLES_PER_DAY: u32 = 3;
pub const MAX_TOTAL_APPLES_PER_DAY: u32 = 5;

pub const RED_APPLE_EXP: u64 = 10;
pub const GOLDEN_APPLE_EXP: u64 = 25;

pub const REST_QUIET_DAYS: u32 = 14;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GrowthStage {
    EyasChick,
    SkyFledgling,
    PeregrineSwift,
    ApexCometFalcon,
}

impl GrowthStage {
    pub fn from_level(level: u32) -> Self {
        match level {
            0..=5 => Self::EyasChick,
            6..=15 => Self::SkyFledgling,
            16..=30 => Self::PeregrineSwift,
            _ => Self::ApexCometFalcon,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::EyasChick => "EYAS",
            Self::SkyFledgling => "FLEDGLING",
            Self::PeregrineSwift => "PEREGRINE",
            Self::ApexCometFalcon => "APEX COMET",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Biome {
    EmeraldValley,
    CrimsonCanyon,
    NeonSkyline,
    CometStratosphere,
}

impl Biome {
    pub fn from_distance(distance_km: u64) -> Self {
        match distance_km {
            0..=499 => Self::EmeraldValley,
            500..=1_999 => Self::CrimsonCanyon,
            2_000..=4_999 => Self::NeonSkyline,
            _ => Self::CometStratosphere,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::EmeraldValley => "EMERALD VALLEY",
            Self::CrimsonCanyon => "CRIMSON CANYON",
            Self::NeonSkyline => "NEON SKYLINE",
            Self::CometStratosphere => "COMET STRATOSPHERE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FalconStats {
    pub level: u32,
    pub total_exp: u64,
    pub xp_in_level: u64,
    pub xp_needed_for_next: u64,
    pub growth_stage: GrowthStage,
    pub distance_km: u64,
    pub biome: Biome,
    pub total_red_apples: u64,
    pub total_golden_apples: u64,
    pub today_red_apples: u32,
    pub today_golden_apples: u32,
    pub recent_red_apples: u32,
    pub recent_golden_apples: u32,
    pub streak_days: u32,
    pub quiet_days: u32,
    pub is_resting: bool,
    pub speed_multiplier: f32,
    pub leetcode_total_solved: u32,
}

pub fn cap_daily_apples(github_commits: u32, leetcode_solves: u32) -> (u32, u32) {
    let golden = leetcode_solves.min(MAX_GOLDEN_APPLES_PER_DAY);
    let red = github_commits.min(MAX_RED_APPLES_PER_DAY);
    let golden_final = golden.min(MAX_TOTAL_APPLES_PER_DAY);
    let red_final = red.min(MAX_TOTAL_APPLES_PER_DAY.saturating_sub(golden_final));
    (red_final, golden_final)
}

fn exp_for_level(level: u32) -> u64 {
    if level <= 1 {
        0
    } else {
        let n = u64::from(level - 1);
        n * n * 35
    }
}

pub fn level_from_exp(total_exp: u64) -> (u32, u64, u64) {
    let mut level = 1u32;
    while exp_for_level(level + 1) <= total_exp {
        level += 1;
    }
    let current_floor = exp_for_level(level);
    let next_floor = exp_for_level(level + 1);
    (
        level,
        total_exp - current_floor,
        (next_floor - current_floor).max(1),
    )
}

pub fn evaluate_falcon(snapshot: &ActivitySnapshot) -> FalconStats {
    let observed = NaiveDate::parse_from_str(&snapshot.observed_date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let mut all_dates: BTreeSet<&String> = BTreeSet::new();
    for k in snapshot.github_daily.keys() {
        all_dates.insert(k);
    }
    for k in snapshot.leetcode_daily.keys() {
        all_dates.insert(k);
    }

    let mut total_red_apples = 0u64;
    let mut calendar_golden_apples = 0u64;

    for date_str in all_dates {
        if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            if d > observed {
                continue;
            }
        }
        let gh = snapshot.github_daily.get(date_str).copied().unwrap_or(0);
        let lc = snapshot.leetcode_daily.get(date_str).copied().unwrap_or(0);
        let (r, g) = cap_daily_apples(gh, lc);
        total_red_apples += u64::from(r);
        calendar_golden_apples += u64::from(g);
    }

    // Preserve older LeetCode problems solved prior to the calendar window at a steady baseline
    let lifetime_solved = u64::from(snapshot.leetcode_total_solved);
    let veteran_golden = lifetime_solved.saturating_sub(calendar_golden_apples) / 2;
    let total_golden_apples = calendar_golden_apples + veteran_golden;

    let today_key = observed.format("%Y-%m-%d").to_string();
    let (today_red_apples, today_golden_apples) = cap_daily_apples(
        snapshot.github_daily.get(&today_key).copied().unwrap_or(0),
        snapshot.leetcode_daily.get(&today_key).copied().unwrap_or(0),
    );

    // Recent 3 days determine active apple-catching choreography
    let mut recent_red_apples = 0u32;
    let mut recent_golden_apples = 0u32;
    for offset in 0..3u64 {
        if let Some(d) = observed.checked_sub_days(chrono::Days::new(offset)) {
            let key = d.format("%Y-%m-%d").to_string();
            let (r, g) = cap_daily_apples(
                snapshot.github_daily.get(&key).copied().unwrap_or(0),
                snapshot.leetcode_daily.get(&key).copied().unwrap_or(0),
            );
            recent_red_apples += r;
            recent_golden_apples += g;
        }
    }

    // Quiet days from observed_date backward
    let mut quiet_days = 0u32;
    for offset in 0..365u64 {
        let Some(d) = observed.checked_sub_days(chrono::Days::new(offset)) else {
            break;
        };
        let key = d.format("%Y-%m-%d").to_string();
        let gh = snapshot.github_daily.get(&key).copied().unwrap_or(0);
        let lc = snapshot.leetcode_daily.get(&key).copied().unwrap_or(0);
        if gh == 0 && lc == 0 {
            quiet_days += 1;
        } else {
            break;
        }
    }

    // Streak: allow today to still be in progress if yesterday was active
    let start_offset = {
        let gh0 = snapshot.github_daily.get(&today_key).copied().unwrap_or(0);
        let lc0 = snapshot.leetcode_daily.get(&today_key).copied().unwrap_or(0);
        if gh0 > 0 || lc0 > 0 { 0u64 } else { 1u64 }
    };
    let mut streak_days = 0u32;
    for offset in start_offset..(start_offset + 365) {
        let Some(d) = observed.checked_sub_days(chrono::Days::new(offset)) else {
            break;
        };
        let key = d.format("%Y-%m-%d").to_string();
        let gh = snapshot.github_daily.get(&key).copied().unwrap_or(0);
        let lc = snapshot.leetcode_daily.get(&key).copied().unwrap_or(0);
        if gh > 0 || lc > 0 {
            streak_days += 1;
        } else {
            break;
        }
    }

    let is_resting = quiet_days >= REST_QUIET_DAYS;
    let speed_multiplier = if is_resting {
        0.55
    } else {
        (1.0 + (streak_days as f32 * 0.08)).min(2.5)
    };

    let total_exp = total_red_apples * RED_APPLE_EXP + total_golden_apples * GOLDEN_APPLE_EXP;
    let (level, xp_in_level, xp_needed_for_next) = level_from_exp(total_exp);
    let growth_stage = GrowthStage::from_level(level);
    let distance_km = total_red_apples * 4 + total_golden_apples * 12;
    let biome = Biome::from_distance(distance_km);

    FalconStats {
        level,
        total_exp,
        xp_in_level,
        xp_needed_for_next,
        growth_stage,
        distance_km,
        biome,
        total_red_apples,
        total_golden_apples,
        today_red_apples,
        today_golden_apples,
        recent_red_apples,
        recent_golden_apples,
        streak_days,
        quiet_days,
        is_resting,
        speed_multiplier,
        leetcode_total_solved: snapshot.leetcode_total_solved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn single_day_153_commits_is_capped_at_3_red_apples() {
        let mut github_daily = BTreeMap::new();
        github_daily.insert("2026-11-27".to_string(), 153);
        let snap = ActivitySnapshot {
            observed_date: "2026-11-27".to_string(),
            github_daily,
            ..Default::default()
        };
        let stats = evaluate_falcon(&snap);
        assert_eq!(stats.today_red_apples, 3);
        assert_eq!(stats.total_red_apples, 3);
        assert_eq!(stats.total_exp, 30);
    }

    #[test]
    fn combined_daily_total_caps_at_5_apples() {
        let (r, g) = cap_daily_apples(153, 20);
        assert_eq!(g, 3);
        assert_eq!(r, 2);
        assert_eq!(r + g, 5);
    }

    #[test]
    fn multi_year_history_preserves_past_commits_with_daily_cap() {
        let mut github_daily = BTreeMap::new();
        github_daily.insert("2024-03-10".to_string(), 100); // capped to 3
        github_daily.insert("2025-07-15".to_string(), 2);   // 2
        github_daily.insert("2026-11-27".to_string(), 1);   // 1
        let snap = ActivitySnapshot {
            observed_date: "2026-11-27".to_string(),
            github_daily,
            ..Default::default()
        };
        let stats = evaluate_falcon(&snap);
        assert_eq!(stats.total_red_apples, 6);
        assert_eq!(stats.total_exp, 60);
    }

    #[test]
    fn resting_mode_activates_after_14_quiet_days() {
        let mut github_daily = BTreeMap::new();
        github_daily.insert("2026-11-01".to_string(), 3);
        let snap = ActivitySnapshot {
            observed_date: "2026-11-27".to_string(),
            github_daily,
            ..Default::default()
        };
        let stats = evaluate_falcon(&snap);
        assert!(stats.quiet_days >= 14);
        assert!(stats.is_resting);
        assert!((stats.speed_multiplier - 0.55).abs() < f32::EPSILON);
    }

    #[test]
    fn biome_and_growth_stage_advance_with_distance_and_level() {
        assert_eq!(Biome::from_distance(0), Biome::EmeraldValley);
        assert_eq!(Biome::from_distance(1200), Biome::CrimsonCanyon);
        assert_eq!(Biome::from_distance(3100), Biome::NeonSkyline);
        assert_eq!(Biome::from_distance(6500), Biome::CometStratosphere);

        assert_eq!(GrowthStage::from_level(3), GrowthStage::EyasChick);
        assert_eq!(GrowthStage::from_level(12), GrowthStage::SkyFledgling);
        assert_eq!(GrowthStage::from_level(22), GrowthStage::PeregrineSwift);
        assert_eq!(GrowthStage::from_level(35), GrowthStage::ApexCometFalcon);
    }
}
