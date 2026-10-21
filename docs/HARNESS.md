# Comparison harness against C 1oom

Age of Orion ports the rules of [1oom](https://github.com/1oom-fork/1oom) formula by
formula. The harness checks the port against the C original: it takes the yearly saves of a
computer-only 1oom game, and for every year loads the save into Age of Orion, plays one turn
and compares the result with 1oom's save of the next year, field by field. It can also
replay that turn in 1oom and compare every random number both sides draw, which points at
the first rule that differs (both games use the same random number generator, so as long as
the rules agree the sequences are identical).

## What you need

- Your own MOO1 data files (the `.LBX` files of a MOO1 installation). They are only read
  locally and are never committed.
- 1oom built with the harness patch (`tools/harness/1oom-harness.patch`, about 50 lines):
  computer-only games, a save every year, stop after a given year, an optional log of every
  random number with the C source line that drew it.

```sh
tools/harness/build-1oom.sh                    # clones 1oom 42e9c0b into target/1oom and builds 1oom_cmdline
tools/harness/run-1oom.sh ~/moo1 /tmp/oom 200  # a 200-year computer-only game, 1oom_save_YYYY.bin per year
cargo run --release -p aoo-harness -- lockstep /tmp/oom \
    --oom target/1oom/src/1oom_cmdline --data ~/moo1
```

`run-1oom.sh LBXDIR OUTDIR YEARS [NEWGAME] [AI]` takes 1oom's `-new` argument
(`OPT:RACES:BANNERS:GALAXYSEED:HUMANS`, default `410:0:0:12345:0`: small galaxy, easy, three
opponents) and the AI (`0` Classic, `1` Classic+). If `LBXDIR` has no RESEARCH.LBX, one is
written from Age of Orion's tech tables (1oom needs it; the tech groups in it are the ones
Age of Orion uses).

## Output

```
2309 -> 2310: 1 fields differ
  rng: first difference at call 40 of 68/66
     38: 1oom 10 9 game_ai_classic.c:2140   | aoo 10 9 crates/core/src/combat.rs:239
    >40: 1oom 10 1 game_ai_classic.c:2140   | aoo 100 1 crates/core/src/spy.rs:342
  seed: 1oom 2316177981 | aoo 1224025442
...
37 of 39 turns identical
```

Each line of the random number list is `range result source`: here 1oom kept resolving a
battle between computer empires where Age of Orion had already moved on to espionage. The
field list uses the same keys on both sides (`p12.pop`, `e3.t2.invest`, `e1.d4` for a ship
design, `fleet7`, `evn.comet`, ...); `--show N` prints more of them.

Other commands: `aoo-harness dump SAVE` prints a decoded 1oom save as JSON,
`aoo-harness import SAVE` prints it as an Age of Orion save (loadable in the game),
`aoo-harness step SAVE NEXT` compares a single turn, `aoo-harness research-lbx OUT` writes the
RESEARCH.LBX described above.

## How the comparison works

- `crates/harness/src/oom.rs` decodes the 1oom save format (`game_save.c`, version 9).
- `import.rs` converts it to an Age of Orion `State`. Values 1oom derives on loading
  (production, ranges, abilities, visibility) are recomputed by `Game::from_state`, the
  equivalent of 1oom's `game_start`.
- `canon.rs` turns both states into the same list of keys and values: every planet's owner,
  population, factories, waste, sliders, build queue, reserve and projects; every empire's
  reserve, taxes, security, research projects and investments, completed techs, ship designs,
  ships in orbit, relations, treaties, moods, trade and spies; the fleets and transports in
  flight; events and monsters; the council.
- The C random number log comes from replaying the same turn with 1oom (`-load`). Ground
  combat draws on a copy of 1oom's seed, so those rolls are in neither log, but the seed
  after them is compared.

## Status

On a small three-opponent game (1oom `-new 410:0:0:12345:0`), every one of the first 39 turns
is identical: production, waste, population growth, research and breakthroughs, the computer
players' scouts, colony ships, attacks, defence, transports, taxes, production sliders, ship
designs and scrapping, battles between computer empires, invasions, events, diplomacy.

Longer games (200 years, 3 to 5 empires, other galaxy sizes) agree on about half of the
turns. The first differences found there are in parts not yet compared rule by rule: some
random events, orbital bombing damage and a branch of the computer players' diplomacy. Once
one turn differs the games drift apart, so each year is compared starting from 1oom's own
save.

Not covered yet: generating the same galaxy as 1oom from the same seed (the harness starts
from 1oom's saves), human players' screens, and tactical battles (the harness games have no
human player).
