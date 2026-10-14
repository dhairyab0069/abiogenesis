# Abiogenesis Simulation

This project simulates the process of abiogenesis(from a layman's perspective) - the process by which life arises from non-living matter. The simulation focuses on simple modeling chemical reactions, molecular interactions, and evolutionary processes in a primordial soup environment.

## Install

Abiogenesis is a single self-contained binary. Nothing else to install: no Java, no runtime, no dependencies.

1. Download the binary for your platform from the [latest release](https://github.com/dhairyab0069/abiogenesis/releases/latest):

   | Platform | File |
   |---|---|
   | macOS (Apple Silicon) | `abiogenesis-macos-arm64` |
   | macOS (Intel) | `abiogenesis-macos-x86_64` |
   | Linux (x86_64) | `abiogenesis-linux-x86_64` |
   | Windows (x86_64) | `abiogenesis-windows-x86_64.exe` |

2. Run it.

   macOS / Linux:
   ```bash
   chmod +x abiogenesis-macos-arm64
   ./abiogenesis-macos-arm64
   ```
   On macOS, a binary downloaded with a browser is quarantined. If it refuses to open, run
   `xattr -d com.apple.quarantine abiogenesis-macos-arm64` once.

   Windows: double-click `abiogenesis-windows-x86_64.exe`.

Optionally move it onto your `PATH` (e.g. `mv abiogenesis-macos-arm64 /usr/local/bin/abiogenesis`).

### Usage

```text
abiogenesis              open the launcher menu
abiogenesis molecules    open the molecule simulation directly
abiogenesis chains       open the amino acid chain generator directly
abiogenesis --version
```

## Project Structure

- `src/`
  - `model.rs` - Atoms, positions, molecules and amino acid chains (mutation, crossover, ...)
  - `soup.rs` - `PrimordialSoup`, the simulation environment with temperature-dependent motion
  - `app.rs` - The GUI ([egui](https://github.com/emilk/egui)): launcher, molecule simulation with glow effects, chain generator
  - `main.rs` - Entry point and command-line arguments

## Implementation Phases

### ✅ Phase 1: Basic Simulation (Toy Model)
- [x] Basic molecular structure representation
- [x] Simple spatial positioning system
- [x] Initial chemical reaction framework
- [x] Temperature and pH environment parameters
- [x] Basic GUI visualization
- [x] Temperature-dependent Brownian motion
- [x] Collision detection
- [x] Energy-based molecule properties
- [x] High frame rate animation (target 60 FPS)
- [x] Mac M1/M2 compatibility
- [x] Interactive tooltips for molecule details in GUI
- [x] Main menu/launcher for simulation selection
- [x] Amino acid chain simulation UI
- [x] Random amino acid chain generation
- [x] Pattern emergence tracking (e.g., "METHINKS")
- [x] Mutation and combination mechanisms

### 🔄 Phase 2: Environmental Dynamics
- [ ] User Controls
- [ ] Grid-based environment implementation
- [ ] Environmental cycles:
  - [ ] Wet/dry cycles
  - [ ] Temperature variations
  - [ ] pH fluctuations
- [ ] Compartmentalization:
  - [ ] Protocell formation
  - [ ] Membrane dynamics
  - [ ] Internal/external environment interactions

### 🧬 Phase 3: Chemistry and Emergence
- [ ] Advanced molecule types:
  - [ ] Amino acids
  - [ ] Lipids
  - [ ] Nucleotides
- [ ] Chemical processes:
  - [ ] Polymerization rules
  - [ ] Autocatalysis
  - [ ] Self-replication mechanisms
- [ ] Fitness functions:
  - [ ] Stability metrics
  - [ ] Replication efficiency
  - [ ] Environmental adaptation

### 🔍 Phase 4: Visualization & Metrics
- [ ] Enhanced visualization:
  - [ ] Real-time molecule distribution
  - [ ] Environmental state display
  - [ ] Chemical reaction visualization
- [ ] Metrics tracking:
  - [ ] Molecular diversity
  - [ ] Complexity measures
  - [ ] Self-replicator population
  - [ ] Environmental statistics

### 🧠 Phase 5: Advanced Concepts
- [ ] GARD model implementation
- [ ] Information theory metrics:
  - [ ] Entropy calculations
  - [ ] Mutual information
  - [ ] Information flow
- [ ] Evolutionary dynamics:
  - [ ] Population genetics
  - [ ] Selection pressures
  - [ ] Adaptation mechanisms

## Building from Source

Requires a [Rust toolchain](https://rustup.rs) (1.95+).

```bash
cargo run --release            # build and run
cargo test                     # run the tests
cargo build --release          # binary at target/release/abiogenesis
```

Releases are built by GitHub Actions: pushing a tag like `v0.2.0` builds the binaries for all
platforms and attaches them to a GitHub release.

## Current Features (Phase 1 Complete)

- **Amino Acid Generation Only:**
  - The simulation currently generates random amino acid chains in a 2D environment.
  - Generation is capped (max 50 chains) and occurs at a slow, controlled rate.
  - No degradation or reactions are active in this phase.
  - **Population is now stably controlled:** Mutations always replace the original chain (no loss), and the cap is enforced only for generation and combination.

- **Pattern Emergence Tracking:**
  - The simulation tracks the appearance of specific amino acid sequences (e.g., "METHINKS") in the population.

- **Mutation and Combination Mechanisms:**
  - Amino acid chains can undergo mutation (substitution, insertion, deletion) and recombination (crossover), simulating evolutionary processes.
  - **Mutation is robust:** Each mutation replaces the original chain, so the population remains stable.
  - **Combination is limited:** Only a small number of new chains are created per step, and only if under the population cap.

- **Main Menu / Launcher:**
  - Users can select between the main simulation and the amino acid chain simulation UI from a launcher window.

- **Amino Acid Chain Simulation UI:**
  - Dedicated interface for generating and inspecting random amino acid chains.

- **Visualization:**
  - Native GUI (egui) displays molecules and amino acid chains as glowing colored circles.
  - Color is based on chain length.
  - Interactive tooltips show sequence, length, energy, and atomic composition.
  - Live statistics (FPS, step, molecule counts, temperature, pH) and pattern sightings in a side panel, with pause, single-step and reset controls.
  - Window resizing is supported, with molecule positions rescaled accordingly.

- **Architecture:**
  - Modular, layered, and MVC-inspired for easy extension.
  - Codebase is ready for future phases (e.g., enabling reactions, degradation, environmental effects).

- **Documentation:**
  - Up-to-date README and detailed feature documentation in `docs/phase1_features.md`.

## Next Steps
- Enable molecule reactions and degradation in future phases.
- Add environmental dynamics and evolutionary features.

---

For more details, see `docs/phase1_features.md`.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Project Architecture

This project uses a **modular, layered architecture** with elements of the **MVC (Model-View-Controller) pattern**:

- **Presentation Layer (View/UI):**
  - `app.rs` (launcher, `SoupView`, `ChainsView`) handles user interaction and visualization.
- **Business Logic Layer (Simulation):**
  - Simulation logic (`soup.rs`, `PrimordialSoup`) manages the rules, state, and processes of the simulation.
- **Data/Model Layer:**
  - `model.rs` (`Atom`, `Molecule`, `Position`, chain generation) represents the core data structures and utilities.

**Summary Table:**

| Layer         | Package/Files                        | Responsibility                                 |
|---------------|-------------------------------------|------------------------------------------------|
| GUI           | src/app.rs                          | User interface, visualization, simulation selection |
| Simulation    | src/soup.rs                         | Simulation logic for molecules/environment     |
| Model         | src/model.rs                        | Data structures and utilities                  |
| Build/Config  | Cargo.toml, .github/workflows/, docs/ | Build, release, documentation                |

**Classification:**
> The project is a **modular, layered application** with strong elements of the **MVC pattern**. This makes it robust, extensible, and easy to maintain or extend with new simulation types. 
