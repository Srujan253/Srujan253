mod collect;
mod encode;
mod render;
mod rpg;
mod sprites;

use std::env;
use std::fs;
use std::path::PathBuf;

use collect::{collect_snapshot, synthetic_snapshot, ActivitySnapshot};
use encode::{write_gif, write_png};
use render::{render_frame, FRAME_COUNT};
use rpg::{evaluate_falcon, FalconStats};
use serde::Serialize;

#[derive(Serialize)]
struct SavedState {
    snapshot: ActivitySnapshot,
    stats: FalconStats,
}

struct CliArgs {
    github_user: String,
    leetcode_user: Option<String>,
    output_dir: PathBuf,
    synth: bool,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut github_user = String::from("Srujan253");
    let mut leetcode_user: Option<String> = None;
    let mut output_dir = PathBuf::from("falcon");
    let mut synth = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--github-user" => {
                github_user = args
                    .next()
                    .ok_or_else(|| "--github-user requires a value".to_string())?;
            }
            "--leetcode-user" => {
                leetcode_user = Some(
                    args.next()
                        .ok_or_else(|| "--leetcode-user requires a value".to_string())?,
                );
            }
            "--output-dir" => {
                output_dir = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--output-dir requires a value".to_string())?,
                );
            }
            "--synth" => {
                synth = true;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(CliArgs {
        github_user,
        leetcode_user,
        output_dir,
        synth,
    })
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = parse_args()?;
    fs::create_dir_all(&cli.output_dir)
        .map_err(|e| format!("create {}: {e}", cli.output_dir.display()))?;

    let state_path = cli.output_dir.join("state.json");
    let previous_snapshot: Option<ActivitySnapshot> = fs::read_to_string(&state_path)
        .ok()
        .and_then(|txt| serde_json::from_str::<serde_json::Value>(&txt).ok())
        .and_then(|v| serde_json::from_value(v["snapshot"].clone()).ok());

    let snapshot = if cli.synth {
        synthetic_snapshot()
    } else {
        let token = env::var("GITHUB_TOKEN").ok();
        collect_snapshot(
            &cli.github_user,
            cli.leetcode_user.as_deref(),
            token.as_deref(),
            previous_snapshot.as_ref(),
        )?
    };

    let stats = evaluate_falcon(&snapshot);
    let frames: Vec<_> = (0..FRAME_COUNT)
        .map(|f| render_frame(&stats, f))
        .collect();

    let gif_path = cli.output_dir.join("falcon.gif");
    let png_path = cli.output_dir.join("falcon.png");
    write_gif(&gif_path, &frames, stats.speed_multiplier)?;
    write_png(&png_path, &frames[17])?;

    let saved = SavedState { snapshot, stats };
    let json_bytes = serde_json::to_string_pretty(&saved)
        .map_err(|e| format!("serialize state error: {e}"))?;
    fs::write(&state_path, json_bytes)
        .map_err(|e| format!("write {}: {e}", state_path.display()))?;

    println!(
        "git-falcon rendered: LV.{} {} | {} KM | {} | red={} gold={}",
        saved.stats.level,
        saved.stats.growth_stage.label(),
        saved.stats.distance_km,
        saved.stats.biome.label(),
        saved.stats.total_red_apples,
        saved.stats.total_golden_apples,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_render_produces_valid_gif_and_png() {
        let snap = synthetic_snapshot();
        let stats = evaluate_falcon(&snap);
        let frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_frame(&stats, f))
            .collect();
        assert_eq!(frames.len(), FRAME_COUNT);

        let tmp = env::temp_dir().join("git_falcon_test");
        let _ = fs::create_dir_all(&tmp);
        let gif_p = tmp.join("test.gif");
        let png_p = tmp.join("test.png");
        write_gif(&gif_p, &frames, stats.speed_multiplier).expect("gif write");
        write_png(&png_p, &frames[0]).expect("png write");

        let gif_bytes = fs::read(&gif_p).expect("read gif");
        assert!(gif_bytes.starts_with(b"GIF89a"));
        assert!(gif_bytes.len() > 10_000);

        let png_bytes = fs::read(&png_p).expect("read png");
        assert!(png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
