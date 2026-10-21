# MOO1 quirks kept on purpose

Age of Orion follows 1oom's rules, and 1oom keeps several bugs of Master of Orion 1.3
because they are part of how the game plays (1oom marks them `WASBUG` or `FIXME BUG` and
offers fixes behind options that are off by default). This page lists the ones a player can
notice, so nobody "fixes" them by accident. Every entry names the 1oom source and where the
rule lives here; the comparison harness (`docs/HARNESS.md`) checks them against C 1oom.

## Combat Transporters and the Sub-Space Interdictor

What MOO1 meant: an empire with **Combat Transporters** (Propulsion 45) saves some of the
transports that missile bases and ships shoot down, half of them, a quarter if the planet's
owner has a **Sub-Space Interdictor** (Propulsion 43).

What MOO1 does (1oom `game_turn_transport`, `game_num_combat_trans_fix = false`): the saving
throw only happens when the owner of the target planet **has** the Interdictor. Then each
transport shot down still lands if a 1-in-4 roll succeeds, or else a 1-in-2 roll: 62.5% of
them. Against a planet without an Interdictor, Combat Transporters do nothing.

The Interdictor has no other effect in MOO1, so researching it only helps enemies invading
you. The research screen says so in its description of both techs.

Here: `crates/core/src/ground.rs` (`land`); descriptions: `effect.combat_transporters` and
`effect.sub_space_interdictor` in the text files.

## Transports and Nuclear Engines

Transports travel at the warp of the best engine known, minus one step: Nuclear Engines
(Propulsion 6) leave them at warp 1; they speed up from Sub-Light Drives on
(1oom `game_update_tech_util`, `have_engine = (tech - 6) / 6 + 1`).

Here: `crates/core/src/rules.rs` (`engine_warp`).

## Redirecting fleets

A fleet can change destination while it is still at the planet it leaves; once it has left,
only with **Hyperspace Communications** (Computers 34). Transports in flight can change
destination only with Hyperspace Communications, to a planet within fuel range
(1oom `ui_starmap_enroute`, `ui_starmap_transport`). A redirected fleet keeps its speed
(`game_num_stargate_redir_fix = false`); to avoid the MOO1 exploit of redirecting a fleet that
travels through a stargate to anywhere in one turn, such fleets cannot be redirected here.

Here: `crates/core/src/commands.rs` (`RedirectFleet`, `RedirectTransport`).

## Diplomacy

- Breaking a treaty raises the breaker's trust and blames both empires in Classic; Classic+
  lowers it and blames only the breaker (1oom `game_diplo_break_treaty`).
- The rule meant to make an empire leave two alliances with empires at war with each other
  never fires: its loop compares an empire's treaty with itself
  (1oom `game_ai_classic_turn_diplo_p1_sub1`).
- In Classic, whether two computer empires make peace depends on the variant of the last
  audience message shown (1oom `evn.diplo_msg_subtype`).

Here: `crates/core/src/diplo.rs`, `crates/core/src/ai_diplo.rs`.

## Bombing

- The best battle computer among **all** the attacker's designs counts, even designs with no
  ships over the planet.
- The fourth weapon slot of a design never bombs.
- Biological weapons lower the planet's maximum population even when the attacker chooses
  not to bomb (`game_num_orbital_bio_fix = false`).

Here: `crates/core/src/combat.rs` (`bomb_damage`, `bomb`); 1oom `game_turn_bomb_damage`.

## Comets and pirates

Ships fighting a comet or pirates roll damage for every design with the hull of the first
design (1oom `game_event_run` never advances its design pointer).

Here: `crates/core/src/events.rs`.

## Fixed in MOO1 v1.3a and always on

1oom applies these fixes of the official 1.3a patch, and so does Age of Orion: up to 2700
factories, the "four colony ships" curse of the computer players, the Oracle Interface,
orbital weapons and torpedoes, ships scanning, and the Guardian's specials and repair
(1oom `game_num_fix_bugs_1_3a`).
