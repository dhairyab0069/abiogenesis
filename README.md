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

It is built in six stages. Stages 1 to 5 explain how the process works and how its parts interact, in
plain terms, from molecules in a pool up to cells and multicellular life. Stage 6 is a scenario lab:
set up conditions, pick natural or forced selection, and watch how a population evolves over time.

**Where it is today:** stages 1 to 5 are done. Stage 6 is described in the [roadmap](#roadmap).

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
abiogenesis formation    open the genes and proteins scene directly
abiogenesis cells        open the cells and colonies demo directly
abiogenesis --version
```

**The molecule simulation.** Water, methane and amino acid chains drift in a 2D soup with
temperature-dependent Brownian motion. Hover a molecule for its sequence, length, energy and atomic
composition. The side panel shows live stats (FPS, step, molecule counts, temperature, pH, pool level,
protocells) and any sightings of the target pattern, with pause, single-step and reset controls.

**The environment (stage 2).** The row under the controls picks what the soup feels over time:
*Constant* (the stage 1 soup), *Day and night* (temperature and pH swing in opposite directions every
20 seconds), *Wet and dry pools* (the pool dries out and refills every 40 seconds) or both, with sliders
for the base temperature and pH. As the pool dries the water line sinks, stranded molecules settle into
what is left, and chains crowd together. Where three or more share a grid cell, a **protocell** forms: a
droplet that keeps its chains in, where they mutate twice as fast and cross over with each other. When
the pool refills, the droplets burst.

**The chemistry (stage 3).** The soup now also holds lipids and free nucleotides (the four RNA letters
A, C, G and U). Two nucleotides that meet join into an RNA-like *strand*, and a strand picks up more as
they drift by, up to 24 long; drying makes bonding more likely and wetness makes strands fall apart
faster. A strand copies itself when enough free nucleotides are within reach: the copy is its
complement (A pairs with U, C with G) with the odd wrong base, and it uses up the building blocks, so
strands compete for them. Every molecule has a **fitness** from 0 to 1 (shown in its tooltip): for a
strand it is how balanced its G and C are (strong enough to last, loose enough to copy), how long it
is, plus a bonus for carrying the catalytic motif `GGU`, which makes it a *catalyst* that copies
itself 4 times as fast. Fit strands last longer, so over time the soup fills with strands that are
better at surviving and copying. Lipids cling together, and where they are present a droplet forms with
fewer chains and grows wider. Inside a droplet a strand falls apart half as fast and copies itself
twice as fast. The side panel shows nucleotides, strands, catalysts, the longest strand, the mean
strand fitness and how many strands were copied and lost.

**Genes and proteins (stage 4).** A strand that contains the start codon `AUG` carries a *gene*: read
three bases at a time with the real genetic code, up to a stop codon, it spells a protein. Now and
then such a strand makes its protein, which appears as a new amino acid chain next to it (3 times as
often inside a protocell; a full soup only takes a new protein in place of its least fit chain, and
only if the newcomer is fitter). And it pays back: a strand whose protein is fit copies itself up to
twice as fast. Strands with a gene wear a small green dot, their tooltip shows the protein, and the
side panel counts genes, proteins made and the mean protein fitness. Hover a chain that came from a
gene to see where it was read from.

**Charts and lineage (stage 4).** Every 30 steps the soup records a sample (strands, catalysts, mean
strand fitness, proteins made, how many different strand sequences there are, temperature, pool level
and protocells; the last 400 samples are kept). The side panel draws each as a small chart. The
*Protein lineage* section lists the biggest family of protein chains: the protein read off a strand,
then every chain that mutated or crossed over from it. (Strands themselves have no ids, so the lineage
follows protein chains, which already have a full event log.)

**The Genes and Proteins screen (stage 4).** An animated, looping scene, also reachable from the
launcher: free nucleotides drift in and bond into a strand one at a time; the strand is copied by base
pairing (A with U, C with G, drawn with two or three hydrogen-bond dashes), the pair twists into a
double helix and unzips into two strands; then a ribosome slides along the strand to `AUG`, reads codons
in threes while amino acid beads grow into a chain, releases it at the stop codon and the chain folds
into a coil. The codon, its amino acid and the protein's fitness are shown. Buttons pause and restart
it, set the speed, jump to a phase, switch between RNA (U) and DNA (T), or load a new random gene. The
scene uses the same `complement_strand`, `reading_frame` and `translate` as the soup, so it always
agrees with the simulation.

**Cells to complex life (stage 5).** Every cell is a small, honest simplification, not a biology
lesson. A crowd of lipids closes into a *cell* and draws in the nearest strands. A cell takes in a couple
of nearby lipids, nucleotides and strands per step, makes some membrane of its own, and its strands copy
themselves from its stock of nucleotides. A cell is as big as what it holds, and its fitness is its
strands' mean fitness blended with how well its membrane is stocked. With 8 lipids and 4 strands it may
divide: the strands go alternately to mother and daughter (with the odd wrong base), and the membrane and
nucleotides are shared out. Fitter cells divide more and die less, so selection works on whole cells; a
dead cell hands its contents back to the soup. **Eukaryotes:** a cell at least 12% wider than a free
cell it overlaps may swallow it. The guest becomes a *mitochondrion* (energy: faster copying and
dividing) and the host's strands move into a *nucleus* (they fall apart 60% slower and copy with 70%
fewer errors). A cell with both is a eukaryote and passes both on to its daughters. **Multicellular
life:** when a eukaryote divides, the daughter may stay attached (35% of the time, 80% inside a colony,
up to 8 cells). A colony moves as one body, and its members take roles: *germ* cells do the dividing,
*soma* cells almost never die and never divide, *feeder* cells reach further for food and pass
nucleotides to the others. Each extra role gives the whole colony 20% more dividing and less dying.
Everything is capped (24 cells, 10 strands each, 3 mitochondria, 8 per colony). Cells are drawn as cells: the membrane is a
lipid bilayer of individual glyphs (a round head and two tails; the outer leaflet's heads face out, the
inner leaflet's face in, the tails meet in the middle, with a slight shimmer), or a smooth double line
when a cell is small on screen or the frame's glyph budget is spent. You can watch a cell form: the
lipids that crowded together turn their heads outward and travel into an arc, the arc curls shut, the
inner leaflet appears, and only then does the soft cytoplasm fill and trap the nearby strands. Strands
are bead rows; a nucleus has a double membrane with pores and holds the strands; mitochondria are beans
with a double membrane and cristae. A dividing cell elongates and pinches in two; a swallowing shows the
host's membrane reaching round the guest, closing, and the guest shrinking into a mitochondrion (the
nucleus fades in after it); colony members press together with shared walls, junction marks and a
colour hint for germ, soma and feeder. Click a cell to open the inspector: a large close-up with labels
for membrane, cytoplasm, nucleus, mitochondria and strands, the cell's real numbers and a short note on
how lipids make a bilayer. The side panel counts cells, eukaryotes, colonies and cell fitness, charts
them, and lists the latest cell events. *Cells and Colonies* in the launcher is the molecule simulation
started with a few ready-made cells and a speed slider (the slider is in the molecule simulation too).
In the demo the first colony showed up within about 6,000 steps in all 8 seeds tried (under two minutes at normal speed, well under one at the demo's 3x); from an empty soup it takes a few minutes.

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
- [x] **Stage 2, environment.** Temperature, pH and wetness that cycle over time (wet/dry pools,
  day and night), user controls, then a grid environment and protocells.
- [x] **Stage 3, chemistry and emergence.** Lipids and nucleotides, polymerization rules,
  autocatalysis, self-replication, and a fitness for each molecule.
- [x] **Stage 4, visualization and metrics.** Proteins read off genes, an animated genes-and-proteins
  scene, live charts for diversity, replicators and the environment, and a protein lineage.
- [x] **Stage 5, cells to complex life.** Protocells that grow and divide, then eukaryotes (one cell
  swallows another, which becomes the mitochondrion, and the DNA gets its own envelope, the nucleus),
  then cells that stay together and specialize into multicellular life.
- [ ] **Stage 6, scenario lab.** Build a scenario, choose natural or forced selection, and watch
  populations evolve over time. A small language model trained on protein sequences, written from
  scratch in Rust, supplies the learned fitness and mutation prior.

Stages 2 to 5 stay deliberately simple. Their job is to show people how it works.

## Building from source

Requires a [Rust toolchain](https://rustup.rs) (1.95+).

```bash
cargo run --release            # build and run
cargo test                     # run the tests
cargo build --release          # binary at target/release/abiogenesis
```

Releases are built by GitHub Actions: pushing a tag like `v0.3.0` builds the binaries for all
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
    cell_draw.rs    cells: lipid bilayer, nucleus, mitochondria, formation, pinch, cup, colonies
    cell_inspector.rs  the click-to-inspect close-up
    formation_view.rs  genes and proteins screen: draws a Formation snapshot
    sparkline.rs    painter-drawn line charts for the side panel
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
    chemistry.rs    stage 3 chemistry: strands, copying, decay
    protein.rs      genes read into proteins, the gene boost
    history.rs      the sampled history behind the charts
    cell.rs         stage 5: Cell, roles, and the pure rules (size, fitness, rates)
    cells.rs        stage 5 in the soup: forming, eating, dividing, swallowing, colonies
  model/      atoms, molecules, amino acid chains
    atom.rs, position.rs  Atom and its elements, Position
    molecule.rs     Kind and Molecule: constructors and queries
    editing.rs      combine, degrade, mutation and crossover
    genetic_code.rs the standard codon table, reading frames, translate
    sequence.rs     the amino acid alphabet and random sequences
  formation/  egui-free timeline of the genes and proteins scene (phases, snapshots, random genes)
  cellviz/    egui-free geometry and timelines for the cell drawings (bilayer, formation, pinch, cup)
  events.rs       the chain event log and protein lineage
  environment.rs  temperature, pH and wetness, optionally cycling
docs/
  phase1_features.md   what stage 1 does, in detail
Cargo.toml    dependencies (eframe, rand) and release profile
```

## Contributing

Contributions are welcome. Please open an issue first to discuss major changes, then send a pull
request.
