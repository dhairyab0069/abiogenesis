# Abiogenesis Simulation

This project simulates the process of abiogenesis - the natural process by which life arises from non-living matter. The simulation focuses on modeling chemical reactions, molecular interactions, and evolutionary processes in a primordial soup environment.

## Project Structure

- `src/main/java/com/abiogenesis/`
  - `model/` - Contains basic model classes
    - `Atom.java` - Represents atomic elements
    - `Molecule.java` - Represents chemical molecules
    - `Position.java` - Represents spatial coordinates
  - `simulation/` - Contains simulation logic
    - `PrimordialSoup.java` - Main simulation environment
  - `gui/` - Contains visualization components
    - `SimulationView.java` - JavaFX-based visualization
  - `Main.java` - Entry point for the simulation

## Implementation Phases

### ✅ Phase 1: Basic Simulation (Toy Model)
- [x] Basic molecular structure representation
- [x] Simple spatial positioning system
- [x] Initial chemical reaction framework
- [x] Temperature and pH environment parameters
- [x] Basic GUI visualization
- [ ] Random amino acid chain generation
- [ ] Pattern emergence tracking (e.g., "METHINKS")
- [ ] Mutation and combination mechanisms

### 🔄 Phase 2: Environmental Dynamics
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
- JavaFX 17.0.2 or higher

## Building and Running

1. Clone the repository
2. Build the project:
   ```bash
   mvn clean install
   ```
3. Run the simulation:
   ```bash
   mvn javafx:run
   ```

## Current Features

- Basic molecular simulation with H2O and CH4 molecules
- Real-time visualization using JavaFX
- Simple chemical reaction framework
- Spatial positioning and collision detection
- Temperature and pH environment parameters

## Future Development

The project is actively being developed according to the phase plan above. Each phase builds upon the previous one, gradually increasing the complexity and realism of the simulation.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change. 