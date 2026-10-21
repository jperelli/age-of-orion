//! Comparison harness against C 1oom (see docs/HARNESS.md).
//!
//! `lockstep DIR` takes the yearly saves of a computer-only 1oom game
//! (`tools/harness/run-1oom.sh`), and for every year imports the save, plays one turn in
//! Age of Orion and compares the result with 1oom's next save. With `--oom BIN --data
//! DIR` it also replays that turn in 1oom to compare every random number drawn, which
//! points at the first rule that differs.

mod canon;
mod import;
mod oom;
mod research;

use std::path::{Path, PathBuf};
use std::process::Command;

use aoo_core::{trace_start, trace_take, Game};

struct Opts {
    oom: Option<PathBuf>,
    data: Option<PathBuf>,
    from: u32,
    to: u32,
    show: usize,
}

fn load(path: &Path) -> oom::Game {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    oom::load(&data).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Replays one 1oom turn from `save` and returns its random number calls.
fn oom_rng(opts: &Opts, save: &Path, year: u16) -> Option<Vec<(u16, u16, String)>> {
    let (bin, data) = (opts.oom.as_ref()?, opts.data.as_ref()?);
    let dir = std::env::temp_dir().join(format!("aoo-harness-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).ok()?;
    let log = dir.join("rng.txt");
    let status = Command::new(bin)
        .args(["-cro", "-nolog", "-skipintro", "-user"])
        .arg(&dir)
        .arg("-data")
        .arg(data)
        .arg("-load")
        .arg(save)
        .env("OOM_HARNESS_YEARS", (year + 1).to_string())
        .env("OOM_HARNESS_RNGLOG", &log)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    let _ = status;
    let text = std::fs::read_to_string(&log).ok()?;
    let _ = std::fs::remove_dir_all(&dir);
    Some(
        text.lines()
            .filter_map(|l| {
                let mut it = l.split(' ');
                Some((
                    it.next()?.parse().ok()?,
                    it.next()?.parse().ok()?,
                    it.next()?.to_string(),
                ))
            })
            .collect(),
    )
}

struct StepResult {
    diffs: Vec<(String, String, String)>,
    rng_note: Option<String>,
}

fn step(opts: &Opts, save: &Path, next: &Path) -> StepResult {
    let g0 = load(save);
    let g1 = load(next);
    let mut game = Game::from_state(import::state(&g0));
    trace_start();
    game.end_turn_auto();
    let ours = trace_take();
    let diffs = canon::diff(&canon::from_oom(&g1), &canon::from_state(game.state()));
    let rng_note = oom_rng(opts, save, g0.year).map(|theirs| {
        let first = theirs
            .iter()
            .zip(ours.iter())
            .position(|(a, b)| a.0 != b.n || a.1 != b.r);
        match first {
            None if theirs.len() == ours.len() => format!("rng: {} calls, identical", ours.len()),
            None => format!(
                "rng: identical for {} calls, then 1oom has {} and we have {}",
                theirs.len().min(ours.len()),
                theirs.len(),
                ours.len()
            ),
            Some(i) => {
                let mut s = format!(
                    "rng: first difference at call {i} of {}/{}\n",
                    theirs.len(),
                    ours.len()
                );
                for k in i.saturating_sub(4)..(i + 3).min(theirs.len().max(ours.len())) {
                    let a = theirs
                        .get(k)
                        .map_or("-".to_string(), |c| format!("{} {} {}", c.0, c.1, c.2));
                    let b = ours.get(k).map_or("-".to_string(), |c| {
                        format!("{} {} {}:{}", c.n, c.r, c.at.file(), c.at.line())
                    });
                    s += &format!(
                        "  {}{k}: 1oom {a:<40} | aoo {b}\n",
                        if k == i { '>' } else { ' ' }
                    );
                }
                s
            }
        }
    });
    StepResult { diffs, rng_note }
}

fn saves(dir: &Path) -> Vec<(u32, PathBuf)> {
    let mut v: Vec<(u32, PathBuf)> = std::fs::read_dir(dir)
        .expect("read dir")
        .filter_map(|e| {
            let p = e.ok()?.path();
            let name = p.file_name()?.to_str()?.to_string();
            let year = name
                .strip_prefix("1oom_save_")?
                .strip_suffix(".bin")?
                .parse()
                .ok()?;
            Some((year, p))
        })
        .collect();
    v.sort();
    v
}

fn print_step(year: u32, r: &StepResult, show: usize) {
    println!("{year} -> {}: {} fields differ", year + 1, r.diffs.len());
    if let Some(n) = &r.rng_note {
        println!("  {}", n.trim_end().replace('\n', "\n  "));
    }
    for (k, a, b) in r.diffs.iter().take(show) {
        println!("  {k}: 1oom {a} | aoo {b}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = Opts {
        oom: None,
        data: None,
        from: 0,
        to: u32::MAX,
        show: 12,
    };
    let mut pos = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--oom" => opts.oom = it.next().map(PathBuf::from),
            "--data" => opts.data = it.next().map(PathBuf::from),
            "--from" => opts.from = it.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--to" => opts.to = it.next().and_then(|v| v.parse().ok()).unwrap_or(u32::MAX),
            "--show" => opts.show = it.next().and_then(|v| v.parse().ok()).unwrap_or(12),
            _ => pos.push(a.clone()),
        }
    }
    match pos.first().map(String::as_str) {
        Some("research-lbx") => {
            let out = PathBuf::from(pos.get(1).map(String::as_str).unwrap_or("RESEARCH.LBX"));
            std::fs::write(&out, research::research_lbx()).expect("write");
            println!("wrote {}", out.display());
        }
        Some("dump") => {
            let g = load(Path::new(&pos[1]));
            println!("{}", serde_json::to_string_pretty(&g).expect("json"));
        }
        Some("import") => {
            let g = load(Path::new(&pos[1]));
            let game = Game::from_state(import::state(&g));
            println!("{}", game.save());
        }
        Some("step") => {
            let r = step(&opts, Path::new(&pos[1]), Path::new(&pos[2]));
            print_step(0, &r, opts.show);
        }
        Some("lockstep") => {
            let list = saves(Path::new(&pos[1]));
            let mut identical = 0;
            let mut total = 0;
            for w in list.windows(2) {
                let (y0, p0) = &w[0];
                let (y1, p1) = &w[1];
                if *y1 != y0 + 1 || *y0 < opts.from || *y0 > opts.to {
                    continue;
                }
                let r = step(&opts, p0, p1);
                total += 1;
                if r.diffs.is_empty() {
                    identical += 1;
                }
                print_step(*y0, &r, opts.show);
            }
            println!("{identical} of {total} turns identical");
        }
        _ => {
            eprintln!("usage: aoo-harness research-lbx OUT | dump SAVE | import SAVE | step SAVE NEXT | lockstep DIR [--oom BIN --data DIR --from Y --to Y --show N]");
            std::process::exit(2);
        }
    }
}
