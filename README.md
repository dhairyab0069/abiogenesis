# Abiogenesis Simulation

This project simulates the process of abiogenesis(from a layman's perspective) - the natural process by which life arises from non-living matter. The simulation focuses on modeling chemical reactions, molecular interactions, and evolutionary processes in a primordial soup environment.

## Project Structure

- `src/main/java/com/abiogenesis/`
  - `model/` - Contains basic model classes
    - `Atom.java` - Represents atomic elements
    - `Molecule.java` - Represents chemical molecules with energy levels
    - `Position.java` - Represents spatial coordinates
  - `simulation/` - Contains simulation logic
    - `PrimordialSoup.java` - Main simulation environment with temperature effects
  - `gui/` - Contains visualization components
    - `SimulationView.java` - JavaFX-based visualization with glow effects
  - `Main.java` - Entry point for the simulation

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
- [ ] Pattern emergence tracking (e.g., "METHINKS")
- [ ] Mutation and combination mechanisms

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

## Requirements

- Java 17 or higher
- Maven 3.6 or higher
- JavaFX 21.0.2 (Mac M1/M2 specific)

## Building and Running

1. Clone the repository
2. Build the project:
   ```bash
   mvn clean package
   ```
3. Run the simulation:
   ```bash
   mvn exec:java -Dexec.mainClass="com.abiogenesis.gui.Launcher"
   ```

## Current Features (Phase 1 In Progress)

- **Amino Acid Generation Only:**
  - The simulation currently generates random amino acid chains in a 2D environment.
  - Generation is capped (max 50 chains) and occurs at a slow, controlled rate.
  - No degradation or reactions are active in this phase.

- **Main Menu / Launcher:**
  - Users can select between the main simulation and the amino acid chain simulation UI from a launcher window.

- **Amino Acid Chain Simulation UI:**
  - Dedicated interface for generating and inspecting random amino acid chains.

- **Visualization:**
  - JavaFX-based GUI displays amino acid chains as colored circles.
  - Color is based on chain length.
  - Interactive tooltips show sequence, length, energy, and atomic composition.
  - Debug statistics (FPS, molecule counts, temperature, pH) are printed to the console.
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
  - JavaFX GUI classes (`Launcher`, `SimulationView`, `AminoAcidChainView`) handle user interaction and visualization.
- **Business Logic Layer (Simulation):**
  - Simulation logic (e.g., `PrimordialSoup`) manages the rules, state, and processes of the simulation.
- **Data/Model Layer:**
  - Classes like `Atom`, `Molecule`, `Position`, and `AminoAcidChainGenerator` represent the core data structures and utilities.

**Summary Table:**

| Layer         | Package/Files                        | Responsibility                                 |
|---------------|-------------------------------------|------------------------------------------------|
| GUI           | gui/Launcher, gui/SimulationView, gui/AminoAcidChainView | User interface, visualization, simulation selection |
| Simulation    | simulation/PrimordialSoup           | Simulation logic for molecules/environment     |
| Model         | model/Atom, model/Molecule, model/Position, model/AminoAcidChainGenerator | Data structures and utilities                  |
| Build/Config  | pom.xml, README.md, docs/           | Build, documentation, and configuration        |

**Classification:**
> The project is a **modular, layered application** with strong elements of the **MVC pattern**. This makes it robust, extensible, and easy to maintain or extend with new simulation types. 
