<h1 align="center">Abiogenesis</h1>

<p align="center">
  A small Rust simulation of how life could arise from non-living matter: molecules drifting in a<br>
  primordial soup, amino acid chains forming, mutating and recombining, and (coming) populations<br>
  evolving under natural and forced selection.
</p>

<p align="center">
  <a href="https://github.com/dhairyab0069/abiogenesis/releases/latest"><b>⬇︎ Download the latest release</b></a>
  &nbsp;·&nbsp; macOS (Apple Silicon and Intel), Linux and Windows &nbsp;·&nbsp; one self-contained binary
</p>

<!-- Hero GIF goes here once the chain visualization is done:
<p align="center">
  <img src="docs/hero.gif" width="720" alt="...">
</p>
-->

<p align="center">
  <a href="https://dhairyab0069.github.io/abiogenesis-io/"><b>Project page</b></a> &nbsp;·&nbsp;
  <a href="docs/phase1_features.md">Phase 1 details</a> &nbsp;·&nbsp;
  <a href="https://github.com/dhairyab0069/abiogenesis/releases">All downloads</a>
</p>

---

## Contents

- [What it is](#what-it-is)
- [Install](#install)
- [Using Abiogenesis](#using-abiogenesis)
- [How it works](#how-it-works)
- [Roadmap](#roadmap)
- [Building from source](#building-from-source)
- [Project layout](#project-layout)
- [Contributing](#contributing)

## What it is

Abiogenesis is a toy model, written from a layperson's point of view, of the step before biology:
simple chemistry turning into things that copy and change. It is meant to be watched and played
with, not to be a research-grade chemistry engine.

It is built in stages. Stages 1 to 4 explain how the process works and how its parts interact, in
plain terms. Stage 5 is a scenario lab: set up conditions, pick natural or forced selection, and
watch how a population evolves over time.

**Where it is today:** stage 1 is done. Stages 2 to 5 are in progress and are described in the
[roadmap](#roadmap).

## Install

Download from the [latest release](https://github.com/dhairyab0069/abiogenesis/releases/latest).
Nothing else is needed: no Java, no runtime, no dependencies. There are installers for macOS and
Windows, and a plain binary for every platform. CI builds the installers and binaries on version
tags.

**Installers**

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `Abiogenesis-<version>-macos-arm64.dmg` |
| macOS (Intel) | `Abiogenesis-<version>-macos-x86_64.dmg` |
| Windows (x86_64) | `Abiogenesis-<version>-windows-x86_64-setup.exe` |

- *macOS:* open the `.dmg` and drag Abiogenesis to Applications. The app is not notarized, so the
  first launch needs right-click, Open (or run
  `xattr -dr com.apple.quarantine /Applications/Abiogenesis.app` once).
- *Windows:* run the setup `.exe`. It installs without admin rights and adds a Start menu entry.
  SmartScreen may warn because the installer is unsigned: click More info, then Run anyway.

**Plain binaries**

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `abiogenesis-macos-arm64` |
| macOS (Intel) | `abiogenesis-macos-x86_64` |
| Linux (x86_64) | `abiogenesis-linux-x86_64` |
| Windows (x86_64) | `abiogenesis-windows-x86_64.exe` |

**macOS and Linux:**

```bash
chmod +x abiogenesis-macos-arm64
./abiogenesis-macos-arm64
```

A binary downloaded with a browser is quarantined on macOS. If it refuses to open, run
`xattr -d com.apple.quarantine abiogenesis-macos-arm64` once.

**Windows:** double-click `abiogenesis-windows-x86_64.exe`.

Optionally move it onto your `PATH`, for example
`mv abiogenesis-macos-arm64 /usr/local/bin/abiogenesis`.

## Using Abiogenesis

```text
abiogenesis              open the launcher menu
abiogenesis molecules    open the molecule simulation directly
abiogenesis chains       open the chain build-up view directly
abiogenesis --version
```

**The molecule simulation.** Water, methane and amino acid chains drift in a 2D soup with
temperature-dependent Brownian motion. Hover a molecule for its sequence, length, energy and atomic
composition. The side panel shows live stats (FPS, step, molecule counts, temperature, pH) and any
sightings of the target pattern, with pause, single-step and reset controls.

**The chain build-up view.** Random amino acid chains appear one colored tile per residue, capped at 50.
Chains mutate (a substitution flashes the changed tile, an insertion pops a new one in, a deletion shrinks
one away) and recombine by crossover (the child briefly shows which part came from which parent). A legend
explains the colors, and there are pause, resume, single-step and reset controls.

## How it works

The code is a small layered app with a model-view split:

| Layer | Files | Responsibility |
|---|---|---|
| GUI | `src/app.rs` | Launcher, molecule and chain views, controls |
| Simulation | `src/soup.rs` | The soup: generation, movement, mutation, recombination |
| Model | `src/model.rs` | Atoms, positions, molecules, chains, and the mutation operators |

Chains are generated up to a cap. A mutation always replaces the original chain, so mutating never
loses a chain, and recombination only adds children while the soup is under the cap. That keeps the
population stable while still letting patterns emerge.

## Roadmap

- [x] **Stage 1, basic simulation.** Molecules, movement, collisions, temperature and pH, amino acid
  chain generation, mutation and recombination, pattern tracking, interactive tooltips, launcher.
- [ ] **Stage 2, environment.** Temperature, pH and wetness that cycle over time (wet/dry pools,
  day and night), user controls, then a grid environment and protocells.
- [ ] **Stage 3, chemistry and emergence.** Lipids and nucleotides, polymerization rules,
  autocatalysis, self-replication, and a fitness for each molecule.
- [ ] **Stage 4, visualization and metrics.** Live charts for molecular diversity, complexity,
  replicators and the environment, plus a clearer view of chain generation and lineage.
- [ ] **Stage 5, scenario lab.** Build a scenario, choose natural or forced selection, and watch
  populations evolve over time. A small language model trained on protein sequences, written from
  scratch in Rust, supplies the learned fitness and mutation prior.

Stages 2 to 4 stay deliberately simple. Their job is to show people how it works.

## Building from source

Requires a [Rust toolchain](https://rustup.rs) (1.95+).

```bash
cargo run --release            # build and run
cargo test                     # run the tests
cargo build --release          # binary at target/release/abiogenesis
```

Releases are built by GitHub Actions: pushing a tag like `v0.2.0` builds the binaries for all
platforms and attaches them to a GitHub release.

## Project layout

```
src/
  main.rs     entry point and command-line arguments
  lib.rs      module list
  app/        the egui GUI
    mod.rs          App, Start, the launcher menu and screen switching
    soup_view.rs    molecule simulation screen: controls, statistics, canvas
    molecule_draw.rs  molecule size, colour and hover text
    chains_view.rs  chain build-up screen: stepping the soup, controls, rows
    tiles.rs        chain build-up drawing: legend, rows of letter tiles, colours
    fps.rs          frames-per-second counter
  chain_view/ egui-free view-model for the build-up animation
    mod.rs          ChainView: sync, advance, rows
    apply.rs        turns logged events into row changes
    row.rs          Row: one chain on screen
    tile.rs         Tile, its motion and animation
    timing.rs       animation durations and the row cap
    hue.rs          residue colours
  soup/       PrimordialSoup, the simulation environment
    mod.rs          the struct, construction, simulate_step
    generation.rs   seed chains and new chains
    mutation.rs     substitution, insertion and deletion
    recombination.rs  crossover of pairs of chains
    movement.rs     Brownian drift and resizing
    census.rs       counts and target-pattern matches
  model/      atoms, molecules, amino acid chains
    atom.rs, position.rs  Atom and its elements, Position
    molecule.rs     Kind and Molecule: constructors and queries
    editing.rs      combine, degrade, mutation and crossover
    sequence.rs     the amino acid alphabet and random sequences
  events.rs       the chain event log
  environment.rs  temperature, pH and wetness, optionally cycling
docs/
  phase1_features.md   what stage 1 does, in detail
Cargo.toml    dependencies (eframe, rand) and release profile
```

## Contributing

Contributions are welcome. Please open an issue first to discuss major changes, then send a pull
request.
