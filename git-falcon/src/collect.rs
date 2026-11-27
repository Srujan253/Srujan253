use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageStat {
    pub name: String,
    pub repo_count: u32,
    pub color_hex: String,
}

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
    #[serde(default)]
    pub languages: Vec<LanguageStat>,
    #[serde(default)]
    pub public_repo_count: u32,
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(20))
        .timeout_connect(Duration::from_secs(10))
        .user_agent("git-falcon/0.2")
        .build()
}

pub fn default_language_color(lang: &str) -> &'static str {
    match lang.to_ascii_lowercase().as_str() {
        "python" => "#3572A5",
        "javascript" => "#f1e05a",
        "typescript" => "#3178c6",
        "rust" => "#dea584",
        "html" => "#e34c26",
        "css" => "#563d7c",
        "java" => "#b07219",
        "c++" | "cpp" => "#f34b7d",
        "c" => "#555555",
        "go" => "#00ADD8",
        "jupyter notebook" => "#DA5B0B",
        "shell" => "#89e051",
        _ => "#a98ff7",
    }
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

    let gh_result = collect_github(&agent, github_user, github_token, today.year());
    let (github_daily, languages, public_repo_count) = match gh_result {
        Ok((days, langs, repos)) if !days.is_empty() => (days, langs, repos),
        Ok(_) | Err(_) => {
            if let Some(prev) = previous {
                eprintln!("warning: using cached GitHub activity from previous snapshot");
                (
                    prev.github_daily.clone(),
                    prev.languages.clone(),
                    prev.public_repo_count,
                )
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
        languages,
        public_repo_count,
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

fn collect_github(
    agent: &ureq::Agent,
    username: &str,
    token: Option<&str>,
    current_year: i32,
) -> Result<(BTreeMap<String, u32>, Vec<LanguageStat>, u32), String> {
    if let Some(tok) = token.filter(|t| !t.trim().is_empty()) {
        if let Ok((days, langs, repos)) = collect_github_graphql(agent, username, tok) {
            if !days.is_empty() {
                return Ok((days, langs, repos));
            }
        }
    }
    let days = collect_github_public_fallback(agent, username, current_year)?;
    let (langs, repos) = collect_github_repos_rest(agent, username).unwrap_or_default();
    Ok((days, langs, repos))
}

fn collect_github_graphql(
    agent: &ureq::Agent,
    username: &str,
    token: &str,
) -> Result<(BTreeMap<String, u32>, Vec<LanguageStat>, u32), String> {
    let meta_query = r#"
        query($login: String!) {
          user(login: $login) {
            contributionsCollection {
              contributionYears
            }
            repositories(first: 100, ownerAffiliations: OWNER, isFork: false, orderBy: {field: PUSHED_AT, direction: DESC}) {
              totalCount
              nodes {
                primaryLanguage {
                  name
                  color
                }
              }
            }
          }
        }
    "#;
    let resp: Value = agent
        .post("https://api.github.com/graphql")
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(json!({
            "query": meta_query,
            "variables": { "login": username }
        }))
        .map_err(|e| format!("GitHub GraphQL meta error: {e}"))?
        .into_json()
        .map_err(|e| format!("GitHub GraphQL JSON parse error: {e}"))?;

    let years: Vec<i32> = resp["data"]["user"]["contributionsCollection"]["contributionYears"]
        .as_array()
        .ok_or_else(|| "missing contributionYears in GitHub GraphQL response".to_string())?
        .iter()
        .filter_map(|v| v.as_i64().map(|y| y as i32))
        .collect();

    let public_repo_count = resp["data"]["user"]["repositories"]["totalCount"]
        .as_u64()
        .unwrap_or(0) as u32;

    let mut lang_counts: HashMap<String, (u32, String)> = HashMap::new();
    if let Some(nodes) = resp["data"]["user"]["repositories"]["nodes"].as_array() {
        for node in nodes {
            if let Some(name) = node["primaryLanguage"]["name"].as_str() {
                let color = node["primaryLanguage"]["color"]
                    .as_str()
                    .unwrap_or_else(|| default_language_color(name))
                    .to_string();
                let entry = lang_counts.entry(name.to_string()).or_insert((0, color));
                entry.0 += 1;
            }
        }
    }

    let mut languages: Vec<LanguageStat> = lang_counts
        .into_iter()
        .map(|(name, (repo_count, color_hex))| LanguageStat {
            name,
            repo_count,
            color_hex,
        })
        .collect();
    languages.sort_by(|a, b| b.repo_count.cmp(&a.repo_count).then(a.name.cmp(&b.name)));
    languages.truncate(5);

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

    Ok((daily, languages, public_repo_count))
}

fn collect_github_repos_rest(
    agent: &ureq::Agent,
    username: &str,
) -> Result<(Vec<LanguageStat>, u32), String> {
    let url = format!("https://api.github.com/users/{username}/repos?per_page=100&type=owner");
    let resp: Value = agent
        .get(&url)
        .call()
        .map_err(|e| format!("GitHub REST repos error: {e}"))?
        .into_json()
        .map_err(|e| format!("GitHub REST repos parse error: {e}"))?;

    let arr = resp
        .as_array()
        .ok_or_else(|| "expected array from GitHub repos REST".to_string())?;

    let mut count = 0u32;
    let mut lang_counts: HashMap<String, u32> = HashMap::new();
    for repo in arr {
        if repo["fork"].as_bool().unwrap_or(false) {
            continue;
        }
        count += 1;
        if let Some(lang) = repo["language"].as_str() {
            *lang_counts.entry(lang.to_string()).or_insert(0) += 1;
        }
    }

    let mut languages: Vec<LanguageStat> = lang_counts
        .into_iter()
        .map(|(name, repo_count)| LanguageStat {
            color_hex: default_language_color(&name).to_string(),
            name,
            repo_count,
        })
        .collect();
    languages.sort_by(|a, b| b.repo_count.cmp(&a.repo_count).then(a.name.cmp(&b.name)));
    languages.truncate(5);

    Ok((languages, count))
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
            12
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
        languages: vec![
            LanguageStat {
                name: "JavaScript".to_string(),
                repo_count: 9,
                color_hex: "#f1e05a".to_string(),
            },
            LanguageStat {
                name: "Python".to_string(),
                repo_count: 6,
                color_hex: "#3572A5".to_string(),
            },
            LanguageStat {
                name: "HTML".to_string(),
                repo_count: 4,
                color_hex: "#e34c26".to_string(),
            },
            LanguageStat {
                name: "Rust".to_string(),
                repo_count: 2,
                color_hex: "#dea584".to_string(),
            },
            LanguageStat {
                name: "CSS".to_string(),
                repo_count: 2,
                color_hex: "#563d7c".to_string(),
            },
        ],
        public_repo_count: 23,
    }
}
