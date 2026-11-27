mod collect;
mod encode;
mod render;
mod render_observatory;
mod render_orbit;
mod rpg;
mod sprites;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use collect::{collect_snapshot, synthetic_snapshot, ActivitySnapshot};
use encode::{write_gif, write_png};
use render::{render_frame, FRAME_COUNT};
use render_observatory::render_observatory_frame;
use render_orbit::render_orbit_frame;
use rpg::{evaluate_falcon, FalconStats};
use serde::Serialize;

#[derive(Serialize)]
struct SavedState {
    snapshot: ActivitySnapshot,
    stats: FalconStats,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Scene {
    Falcon,
    Observatory,
    Orbit,
    All,
}

struct CliArgs {
    github_user: String,
    leetcode_user: Option<String>,
    base_dir: PathBuf,
    output_dir: Option<PathBuf>,
    scene: Scene,
    synth: bool,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut github_user = String::from("Srujan253");
    let mut leetcode_user: Option<String> = None;
    let mut base_dir = PathBuf::from(".");
    let mut output_dir: Option<PathBuf> = None;
    let mut scene = Scene::All;
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
            "--base-dir" => {
                base_dir = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--base-dir requires a value".to_string())?,
                );
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--output-dir requires a value".to_string())?,
                ));
            }
            "--scene" => {
                let val = args
                    .next()
                    .ok_or_else(|| "--scene requires a value".to_string())?;
                scene = match val.as_str() {
                    "falcon" => Scene::Falcon,
                    "observatory" => Scene::Observatory,
                    "orbit" => Scene::Orbit,
                    "all" => Scene::All,
                    _ => return Err(format!("unknown scene: {val}")),
                };
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
        base_dir,
        output_dir,
        scene,
        synth,
    })
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn render_and_save(
    dir: &Path,
    stem: &str,
    frames: &[sprites::Canvas],
    still_idx: usize,
    speed: f32,
) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let gif_path = dir.join(format!("{stem}.gif"));
    let png_path = dir.join(format!("{stem}.png"));
    write_gif(&gif_path, frames, speed)?;
    write_png(&png_path, &frames[still_idx])?;
    Ok(())
}

fn run() -> Result<(), String> {
    let cli = parse_args()?;
    let falcon_dir = cli
        .output_dir
        .clone()
        .unwrap_or_else(|| cli.base_dir.join("falcon"));
    let observatory_dir = cli.base_dir.join("observatory");
    let orbit_dir = cli.base_dir.join("orbit");

    fs::create_dir_all(&falcon_dir)
        .map_err(|e| format!("create {}: {e}", falcon_dir.display()))?;

    let state_path = falcon_dir.join("state.json");
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

    if cli.scene == Scene::Falcon || cli.scene == Scene::All {
        let frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_frame(&stats, f))
            .collect();
        render_and_save(&falcon_dir, "falcon", &frames, 17, stats.speed_multiplier)?;
    }

    if cli.scene == Scene::Observatory || cli.scene == Scene::All {
        let frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_observatory_frame(&stats, f))
            .collect();
        render_and_save(&observatory_dir, "observatory", &frames, 16, 1.0)?;
    }

    if cli.scene == Scene::Orbit || cli.scene == Scene::All {
        let frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_orbit_frame(&stats, f))
            .collect();
        render_and_save(&orbit_dir, "orbit", &frames, 12, stats.speed_multiplier.min(1.5))?;
    }

    let saved = SavedState { snapshot, stats };
    let json_bytes = serde_json::to_string_pretty(&saved)
        .map_err(|e| format!("serialize state error: {e}"))?;
    fs::write(&state_path, json_bytes)
        .map_err(|e| format!("write {}: {e}", state_path.display()))?;

    println!(
        "git-falcon rendered: LV.{} {} | {} KM | {} | langs={}",
        saved.stats.level,
        saved.stats.growth_stage.label(),
        saved.stats.distance_km,
        saved.stats.biome.label(),
        saved.stats.languages.len(),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_render_produces_valid_gifs_for_all_scenes() {
        let snap = synthetic_snapshot();
        let stats = evaluate_falcon(&snap);

        let falcon_frames: Vec<_> = (0..FRAME_COUNT).map(|f| render_frame(&stats, f)).collect();
        let obs_frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_observatory_frame(&stats, f))
            .collect();
        let orbit_frames: Vec<_> = (0..FRAME_COUNT)
            .map(|f| render_orbit_frame(&stats, f))
            .collect();

        let tmp = env::temp_dir().join("git_falcon_multi_scene_test");
        let _ = fs::remove_dir_all(&tmp);
        render_and_save(&tmp.join("falcon"), "falcon", &falcon_frames, 17, 1.0).unwrap();
        render_and_save(&tmp.join("obs"), "observatory", &obs_frames, 16, 1.0).unwrap();
        render_and_save(&tmp.join("orbit"), "orbit", &orbit_frames, 12, 1.0).unwrap();

        for (sub, name) in [
            ("falcon", "falcon"),
            ("obs", "observatory"),
            ("orbit", "orbit"),
        ] {
            let gif_bytes = fs::read(tmp.join(sub).join(format!("{name}.gif"))).unwrap();
            assert!(gif_bytes.starts_with(b"GIF89a"));
            assert!(gif_bytes.len() > 10_000);
            let png_bytes = fs::read(tmp.join(sub).join(format!("{name}.png"))).unwrap();
            assert!(png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }

        let _ = fs::remove_dir_all(&tmp);
    }
}
