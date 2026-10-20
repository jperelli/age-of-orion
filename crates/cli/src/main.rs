//! `aoo`: command line tools for Age of Orion.
//!
//! ```text
//! aoo import <dir with LBX files> <output dir>   convert LBX files to the open asset tree
//! aoo sim [--seed N] [--turns N] [--size S] [--players N] [--save FILE]
//!                                               play a game with AI only and print a summary
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use aoo_core::{GalaxySize, Game, NewGameOptions};

const USAGE: &str = "usage:
  aoo import <lbx-dir> <out-dir>
  aoo sim [--seed N] [--turns N] [--size small|medium|large|huge] [--players N] [--save FILE]";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("import") if args.len() == 3 => import(Path::new(&args[1]), Path::new(&args[2])),
        Some("sim") => sim(&args[1..]),
        Some("--version") => {
            println!("aoo {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn import(src: &Path, out: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    let mut stack = vec![src.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path: PathBuf = e.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("lbx"))
            {
                let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                files.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    bytes,
                ));
            }
        }
    }
    if files.is_empty() {
        return Err(format!("no .LBX files found in {}", src.display()));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let result = aoo_lbx::import(&files);
    for f in &result.files {
        let path = out.join(&f.path);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&path, &f.bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    for w in &result.manifest.warnings {
        eprintln!("warning: {w}");
    }
    println!(
        "imported {} LBX files -> {} assets in {}",
        files.len(),
        result.files.len(),
        out.display()
    );
    Ok(())
}

fn sim(args: &[String]) -> Result<(), String> {
    let opt = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let num = |k: &str, d: u32| -> Result<u32, String> {
        opt(k).map_or(Ok(d), |v| {
            v.parse().map_err(|_| format!("{k}: not a number"))
        })
    };
    let size = match opt("--size").as_deref() {
        None | Some("small") => GalaxySize::Small,
        Some("medium") => GalaxySize::Medium,
        Some("large") => GalaxySize::Large,
        Some("huge") => GalaxySize::Huge,
        Some(s) => return Err(format!("unknown size {s}")),
    };
    let opts = NewGameOptions {
        seed: num("--seed", 1)?,
        galaxy_size: size,
        players: num("--players", 4)? as u8,
        all_ai: true,
        ..NewGameOptions::default()
    };
    let mut game = Game::new(&opts).map_err(|e| e.to_string())?;
    let turns = num("--turns", 200)?;
    for _ in 0..turns {
        if game.state().winner.is_some() {
            break;
        }
        game.end_turn();
        for e in &game.state().events {
            if let aoo_core::EventKind::CouncilResult {
                candidates,
                votes,
                total,
                elected,
            } = &e.kind
            {
                println!(
                    "council {}: {} votes for {}, {} for {} of {total}, elected {elected:?}",
                    game.state().year,
                    votes[0],
                    candidates[0],
                    votes[1],
                    candidates[1]
                );
            }
        }
    }
    let s = game.state();
    println!("year {} ({} turns)", s.year, s.turn);
    for (i, e) in s.empires.iter().enumerate() {
        let colonies = s
            .planets
            .iter()
            .filter(|p| p.owner == Some(i as u8))
            .count();
        let pop: u32 = s
            .planets
            .iter()
            .filter(|p| p.owner == Some(i as u8))
            .map(|p| p.pop as u32)
            .sum();
        println!(
            "empire {i} race {:?}: {} colonies, pop {pop}, tech {:?}, alive {}",
            e.race, colonies, e.tech.level, e.alive
        );
    }
    for (i, e) in s.empires.iter().enumerate() {
        let deals: Vec<String> = e
            .rel
            .iter()
            .enumerate()
            .filter(|(j, _)| *j > i)
            .map(|(j, r)| {
                format!(
                    "{j}:{:?}{}",
                    r.treaty,
                    if r.trade_bc > 0 { "+trade" } else { "" }
                )
            })
            .collect();
        if !deals.is_empty() {
            println!("relations of {i}: {}", deals.join(" "));
        }
    }
    if let Some(w) = s.winner {
        println!("winner: empire {w} ({:?})", s.victory);
    }
    if let Some(path) = opt("--save") {
        fs::write(&path, game.save()).map_err(|e| format!("{path}: {e}"))?;
        println!("saved to {path}");
    }
    Ok(())
}
