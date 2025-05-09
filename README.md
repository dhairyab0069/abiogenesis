# Abiogenesis Simulation

This project simulates the process of abiogenesis - the natural process by which life arises from non-living matter. The simulation focuses on modeling basic chemical reactions and molecular interactions in a primordial soup environment.

## Project Structure

- `src/main/java/com/abiogenesis/`
  - `model/` - Contains basic model classes
    - `Atom.java` - Represents atomic elements
    - `Molecule.java` - Represents chemical molecules
    - `Position.java` - Represents spatial coordinates
  - `simulation/` - Contains simulation logic
    - `PrimordialSoup.java` - Main simulation environment
  - `Main.java` - Entry point for the simulation

## Phase 1 Features

- Basic molecular structure representation
- Simple spatial positioning system
- Initial chemical reaction framework
- Temperature and pH environment parameters

## Requirements

- Java 17 or higher
- Maven 3.6 or higher

## Building and Running

1. Clone the repository
2. Build the project:
   ```bash
   mvn clean install
   ```
3. Run the simulation:
   ```bash
   mvn exec:java -Dexec.mainClass="com.abiogenesis.Main"
   ```

## Future Phases

- Phase 2: Implement more complex chemical reactions
- Phase 3: Add energy transfer and molecular bonding
- Phase 4: Simulate basic metabolic processes
- Phase 5: Introduce self-replicating molecules 