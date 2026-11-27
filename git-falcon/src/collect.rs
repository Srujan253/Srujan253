use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActivitySnapshot {
    pub github_user: String,
    pub leetcode_user: String,
    pub observed_date: String,
    pub github_daily: BTreeMap<String, u32>,
    pub leetcode_daily: BTreeMap<String, u32>,
    pub leetcode_easy: u32,
    pub leetcode_medium: u32,
    pub leetcode_hard: u32,
    pub leetcode_total_solved: u32,
    pub leetcode_ranking: u32,
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(20))
        .timeout_connect(Duration::from_secs(10))
        .user_agent("git-falcon/0.1")
        .build()
}

pub fn collect_snapshot(
    github_user: &str,
    leetcode_user: Option<&str>,
    github_token: Option<&str>,
    previous: Option<&ActivitySnapshot>,
) -> Result<ActivitySnapshot, String> {
    let agent = http_agent();
    let today = Utc::now().date_naive();
    let observed_date = today.format("%Y-%m-%d").to_string();

    let github_daily = match collect_github_daily(&agent, github_user, github_token, today.year()) {
        Ok(days) if !days.is_empty() => days,
        Ok(_) | Err(_) => {
            if let Some(prev) = previous {
                eprintln!("warning: using cached GitHub activity from previous snapshot");
                prev.github_daily.clone()
            } else {
                return Err(format!("failed to collect GitHub contributions for {github_user}"));
            }
        }
    };

    let mut snapshot = ActivitySnapshot {
        github_user: github_user.to_string(),
        leetcode_user: leetcode_user.unwrap_or_default().to_string(),
        observed_date,
        github_daily,
        ..Default::default()
    };

    if let Some(lc_user) = leetcode_user.filter(|s| !s.trim().is_empty()) {
        match collect_leetcode(&agent, lc_user) {
            Ok(lc) => {
                snapshot.leetcode_daily = lc.daily;
                snapshot.leetcode_easy = lc.easy;
                snapshot.leetcode_medium = lc.medium;
                snapshot.leetcode_hard = lc.hard;
                snapshot.leetcode_total_solved = lc.total_solved;
                snapshot.leetcode_ranking = lc.ranking;
            }
            Err(err) => {
                eprintln!("warning: LeetCode fetch failed ({err}); falling back to cached stats");
                if let Some(prev) = previous {
                    snapshot.leetcode_daily = prev.leetcode_daily.clone();
                    snapshot.leetcode_easy = prev.leetcode_easy;
                    snapshot.leetcode_medium = prev.leetcode_medium;
                    snapshot.leetcode_hard = prev.leetcode_hard;
                    snapshot.leetcode_total_solved = prev.leetcode_total_solved;
                    snapshot.leetcode_ranking = prev.leetcode_ranking;
                }
            }
        }
    }

    Ok(snapshot)
}

fn collect_github_daily(
    agent: &ureq::Agent,
    username: &str,
    token: Option<&str>,
    current_year: i32,
) -> Result<BTreeMap<String, u32>, String> {
    if let Some(tok) = token.filter(|t| !t.trim().is_empty()) {
        if let Ok(days) = collect_github_graphql(agent, username, tok) {
            if !days.is_empty() {
                return Ok(days);
            }
        }
    }
    collect_github_public_fallback(agent, username, current_year)
}

fn collect_github_graphql(
    agent: &ureq::Agent,
    username: &str,
    token: &str,
) -> Result<BTreeMap<String, u32>, String> {
    let years_query = r#"
        query($login: String!) {
          user(login: $login) {
            contributionsCollection {
              contributionYears
            }
          }
        }
    "#;
    let resp: Value = agent
        .post("https://api.github.com/graphql")
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(json!({
            "query": years_query,
            "variables": { "login": username }
        }))
        .map_err(|e| format!("GitHub GraphQL years error: {e}"))?
        .into_json()
        .map_err(|e| format!("GitHub GraphQL JSON parse error: {e}"))?;

    let years: Vec<i32> = resp["data"]["user"]["contributionsCollection"]["contributionYears"]
        .as_array()
        .ok_or_else(|| "missing contributionYears in GitHub GraphQL response".to_string())?
        .iter()
        .filter_map(|v| v.as_i64().map(|y| y as i32))
        .collect();

    let year_query = r#"
        query($login: String!, $from: DateTime!, $to: DateTime!) {
          user(login: $login) {
            contributionsCollection(from: $from, to: $to) {
              contributionCalendar {
                weeks {
                  contributionDays {
                    date
                    contributionCount
                  }
                }
              }
            }
          }
        }
    "#;

    let mut daily = BTreeMap::new();
    for year in years {
        let from = format!("{year}-01-01T00:00:00Z");
        let to = format!("{year}-12-31T23:59:59Z");
        let year_resp: Value = agent
            .post("https://api.github.com/graphql")
            .set("Authorization", &format!("Bearer {token}"))
            .send_json(json!({
                "query": year_query,
                "variables": { "login": username, "from": from, "to": to }
            }))
            .map_err(|e| format!("GitHub GraphQL year {year} error: {e}"))?
            .into_json()
            .map_err(|e| format!("GitHub GraphQL year {year} parse error: {e}"))?;

        if let Some(weeks) = year_resp["data"]["user"]["contributionsCollection"]
            ["contributionCalendar"]["weeks"]
            .as_array()
        {
            for week in weeks {
                if let Some(days) = week["contributionDays"].as_array() {
                    for day in days {
                        if let (Some(date), Some(count)) = (
                            day["date"].as_str(),
                            day["contributionCount"].as_u64(),
                        ) {
                            daily.insert(date.to_string(), count as u32);
                        }
                    }
                }
            }
        }
    }

    Ok(daily)
}

fn collect_github_public_fallback(
    agent: &ureq::Agent,
    username: &str,
    _current_year: i32,
) -> Result<BTreeMap<String, u32>, String> {
    let url = format!("https://github-contributions-api.jogruber.de/v4/{username}");
    let resp: Value = agent
        .get(&url)
        .call()
        .map_err(|e| format!("GitHub public contributions fallback failed: {e}"))?
        .into_json()
        .map_err(|e| format!("GitHub public contributions parse error: {e}"))?;

    let mut daily = BTreeMap::new();
    if let Some(arr) = resp["contributions"].as_array() {
        for entry in arr {
            if let (Some(date), Some(count)) =
                (entry["date"].as_str(), entry["count"].as_u64())
            {
                daily.insert(date.to_string(), count as u32);
            }
        }
    }

    Ok(daily)
}

struct LeetCodeData {
    daily: BTreeMap<String, u32>,
    easy: u32,
    medium: u32,
    hard: u32,
    total_solved: u32,
    ranking: u32,
}

fn collect_leetcode(agent: &ureq::Agent, username: &str) -> Result<LeetCodeData, String> {
    let url = format!("https://leetcode-api-faisalshohag.vercel.app/{username}");
    let resp: Value = agent
        .get(&url)
        .call()
        .map_err(|e| format!("LeetCode request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("LeetCode JSON parse failed: {e}"))?;

    let total_solved = resp["totalSolved"]
        .as_u64()
        .ok_or_else(|| "missing totalSolved from LeetCode API".to_string())?
        as u32;
    let easy = resp["easySolved"].as_u64().unwrap_or(0) as u32;
    let medium = resp["mediumSolved"].as_u64().unwrap_or(0) as u32;
    let hard = resp["hardSolved"].as_u64().unwrap_or(0) as u32;
    let ranking = resp["ranking"].as_u64().unwrap_or(0) as u32;

    let mut daily = BTreeMap::new();
    if let Some(cal) = resp["submissionCalendar"].as_object() {
        for (ts_str, count_val) in cal {
            if let (Ok(ts), Some(cnt)) = (ts_str.parse::<i64>(), count_val.as_u64()) {
                if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
                    let date_key = dt.date_naive().format("%Y-%m-%d").to_string();
                    *daily.entry(date_key).or_insert(0) += cnt as u32;
                }
            }
        }
    }

    Ok(LeetCodeData {
        daily,
        easy,
        medium,
        hard,
        total_solved,
        ranking,
    })
}

pub fn synthetic_snapshot() -> ActivitySnapshot {
    let today = Utc::now().date_naive();
    let mut github_daily = BTreeMap::new();
    let mut leetcode_daily = BTreeMap::new();

    for offset in 0..240 {
        let d = today - chrono::Days::new(offset);
        let key = d.format("%Y-%m-%d").to_string();
        let gh = if offset % 5 == 0 {
            12 // tests daily cap of 3
        } else if offset % 2 == 0 {
            2
        } else {
            1
        };
        github_daily.insert(key.clone(), gh);
        if offset % 3 == 0 {
            leetcode_daily.insert(key, 2);
        }
    }

    ActivitySnapshot {
        github_user: "Srujan253".to_string(),
        leetcode_user: "SrujanHM".to_string(),
        observed_date: today.format("%Y-%m-%d").to_string(),
        github_daily,
        leetcode_daily,
        leetcode_easy: 119,
        leetcode_medium: 54,
        leetcode_hard: 4,
        leetcode_total_solved: 177,
        leetcode_ranking: 1_012_416,
    }
}
