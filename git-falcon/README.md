# 🦅 git-falcon

A fast, procedural Rust pixel-art coding pet powered by your **GitHub lifetime contributions** and **LeetCode problem-solving activity**.

## Game Mechanics

- **Daily Anti-Spike Caps**:
  - **Red Apples (GitHub Commits)**: Capped at **3 Red Apples/day** (`min(daily_commits, 3)`), `+10 EXP` each.
  - **Golden Apples (LeetCode Solves)**: Capped at **3 Golden Apples/day**, `+25 EXP` each (with a balanced veteran baseline for older pre-calendar problems).
  - **Daily Total Limit**: At most **5 Apples/day** total (`red + golden <= 5`).
- **Growth Stages**:
  1. **Lv 1–5**: Eyas Chick
  2. **Lv 6–15**: Sky Fledgling
  3. **Lv 16–30**: Peregrine Swift
  4. **Lv 31+**: Apex Comet Falcon
- **Evolving Biomes (by Distance Flown)**:
  - **0–499 km**: Emerald Valley
  - **500–1,999 km**: Crimson Canyon
  - **2,000–4,999 km**: Neon Skyline
  - **5,000+ km**: Comet Stratosphere
- **Flight Speed & Rest**:
  - Consecutive active days (GitHub or LeetCode) scale flight speed from `1.0x` up to `2.5x`.
  - After **14+ quiet days**, the Falcon enters calm dusk **Resting** mode until your next commit or LeetCode solve feeds it an apple.

## Usage

```bash
cargo run --release -- \
  --github-user Srujan253 \
  --leetcode-user SrujanHM \
  --output-dir falcon
```

Run offline synthetic test & unit tests:

```bash
cargo test
cargo run -- --synth --output-dir falcon
```
