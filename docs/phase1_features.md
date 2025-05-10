# Phase 1: All Features Explained

This document provides an in-depth explanation of all features in Phase 1 of the Abiogenesis Simulation project. Each feature is described with its logic, scientific context, and implementation notes.

---

## 0. Main Menu/Launcher for Simulation Selection

**Logic:**
- Provide a starting interface where users can choose between different simulation modes (e.g., molecule simulation or amino acid chain simulation).
- Allow switching between simulations without restarting the application.

**Scientific Context:**
- Facilitates modular exploration of different abiogenesis scenarios and models.
- Encourages comparative study and user-driven experimentation.

**Implementation Notes:**
- Create a main menu window with buttons for each simulation type.
- Implement logic to launch the selected simulation and return to the menu.

---

## 0. Amino Acid Chain Simulation UI

**Logic:**
- Dedicated interface for generating, displaying, and interacting with random amino acid chains.
- Allows users to observe chain properties and regeneration.

**Scientific Context:**
- Enables focused study of polypeptide diversity and sequence statistics.
- Lays groundwork for future integration with the main simulation.

**Implementation Notes:**
- Build a new JavaFX view for amino acid chains.
- Display generated chains, their lengths, and provide controls for regeneration.

---

## 1. Basic Molecular Structure Representation

**Logic:**
- Represent molecules as collections of atoms, each with properties like element type, atomic number, and mass.
- Molecules have a name, energy, and a list of constituent atoms.

**Scientific Context:**
- Molecules are the fundamental units of chemistry.
- Understanding their structure is essential for simulating chemical reactions and emergent properties.

**Implementation Notes:**
- Use classes for `Atom` and `Molecule`.
- Store atoms in a list within each molecule.
- Assign properties such as energy and position.

---

## 2. Simple Spatial Positioning System

**Logic:**
- Each molecule has a position in a 2D environment (x, y coordinates).
- Positions are updated as molecules move.

**Scientific Context:**
- Spatial distribution affects collision rates and reaction likelihood.
- Mimics the random movement of molecules in a fluid.

**Implementation Notes:**
- Use a `Position` class with x and y fields.
- Store and update positions for each molecule.

---

## 3. Initial Chemical Reaction Framework

**Logic:**
- Allow molecules to interact and react when close together and with sufficient energy.
- Example: H2O + CH4 → CH3OH.

**Scientific Context:**
- Chemical reactions are central to abiogenesis, leading to new, more complex molecules.
- Simulates the transformation of simple molecules into more complex ones.

**Implementation Notes:**
- Check for proximity and energy thresholds.
- Remove reactants and add products to the simulation.

---

## 4. Temperature and pH Environment Parameters

**Logic:**
- The environment has a temperature (affecting movement) and pH (potentially affecting reactions).
- These parameters can be adjusted to study their effects.

**Scientific Context:**
- Temperature influences reaction rates and molecular motion.
- pH affects chemical stability and reaction pathways.

**Implementation Notes:**
- Store temperature and pH as fields in the environment class.
- Use temperature to scale Brownian motion.

---

## 5. Basic GUI Visualization

**Logic:**
- Visualize molecules as colored circles on a 2D canvas.
- Use different colors for different molecule types.

**Scientific Context:**
- Visualization helps interpret and analyze simulation results.
- Makes emergent patterns and dynamics observable.

**Implementation Notes:**
- Use JavaFX for rendering.
- Assign colors (e.g., aqua for H2O, green for CH4, purple for CH3OH).

---

## 6. Temperature-Dependent Brownian Motion

**Logic:**
- Molecules move randomly, with speed proportional to temperature.
- Simulates thermal agitation in a fluid.

**Scientific Context:**
- Brownian motion is a fundamental property of particles in a fluid.
- Higher temperatures increase kinetic energy and movement.

**Implementation Notes:**
- Use random number generation scaled by temperature.
- Update positions each simulation step.

---

## 7. Collision Detection

**Logic:**
- Detect when molecules are close enough to interact or react.
- Use distance calculations between positions.

**Scientific Context:**
- Collisions are necessary for chemical reactions to occur.
- Collision frequency affects reaction rates.

**Implementation Notes:**
- Calculate Euclidean distance between molecules.
- Set a threshold for interaction.

---

## 8. Energy-Based Molecule Properties

**Logic:**
- Each molecule has an energy value, affecting its behavior and reactions.
- Energy can influence size in visualization and reaction likelihood.

**Scientific Context:**
- Energy is a key factor in chemical reactivity and stability.
- Simulates the thermodynamics of prebiotic chemistry.

**Implementation Notes:**
- Store energy as a property of each molecule.
- Use energy to scale visual size and check reaction conditions.

---

## 9. High Frame Rate Animation (target 60 FPS)

**Logic:**
- Run the simulation and visualization at a high frame rate for smooth animation.

**Scientific Context:**
- Real-time feedback allows for better observation of dynamic processes.

**Implementation Notes:**
- Use a simulation loop with timing control.
- Target 60 frames per second.

---

## 10. Mac M1/M2 Compatibility

**Logic:**
- Ensure the simulation runs smoothly on Apple Silicon (M1/M2) Macs.

**Scientific Context:**
- Platform compatibility broadens accessibility and usability.

**Implementation Notes:**
- Use the correct JavaFX dependencies and build settings for Mac ARM architecture.

---

## 11. Interactive Tooltips for Molecule Details in GUI

**Logic:**
- Show detailed information about a molecule when hovering over it in the GUI.
- Display type, energy, position, atomic composition, and structure.

**Scientific Context:**
- Enables deeper inspection and understanding of individual molecules and their properties.

**Implementation Notes:**
- Use a single tooltip instance.
- Update and show/hide the tooltip based on mouse position.

---

## 12. Random Amino Acid Chain Generation

**Logic:**
- Generate random sequences of amino acids, simulating prebiotic polypeptide formation.

**Scientific Context:**
- Models the spontaneous creation of diverse polypeptides in early Earth conditions.

**Implementation Notes:**
- Use a list of 20 amino acids.
- Randomly determine chain length and sequence.

---

## 13. Pattern Emergence Tracking (e.g., "METHINKS")

**Logic:**
- Track the appearance of specific sequences in the population of chains.

**Scientific Context:**
- Demonstrates how order and meaningful patterns can emerge from randomness.

**Implementation Notes:**
- Define target patterns.
- Scan chains for matches and collect statistics.

---

## 14. Mutation and Combination Mechanisms

**Logic:**
- Introduce mutations (substitution, insertion, deletion) and recombination (crossover) in chains.

**Scientific Context:**
- Simulates evolutionary processes that drive diversity and adaptation.

**Implementation Notes:**
- Implement mutation and recombination functions.
- Apply at controlled rates to chains.

---

## Summary Table

| Feature                        | Logic Summary                                              | Scientific Context                        |
|---------------------------------|-----------------------------------------------------------|-------------------------------------------|
| Basic molecular structure       | Atoms and molecules with properties                       | Chemistry foundation                      |
| Spatial positioning             | 2D coordinates for molecules                              | Collision, diffusion                      |
| Chemical reaction framework     | Molecules interact/react if close and energetic           | Prebiotic chemistry                       |
| Temperature/pH parameters       | Environmental factors affect simulation                   | Reaction rates, stability                 |
| GUI visualization               | Visual feedback of simulation state                       | Analysis, interpretation                  |
| Brownian motion                 | Random, temperature-dependent movement                    | Thermal agitation                         |
| Collision detection             | Find close molecules for interaction                      | Reaction likelihood                       |
| Energy-based properties         | Energy affects size, reactivity                           | Thermodynamics                            |
| High frame rate animation       | Smooth, real-time simulation                              | Usability                                 |
| Mac M1/M2 compatibility         | Runs on Apple Silicon                                     | Accessibility                             |
| Interactive tooltips            | Inspect molecule details in GUI                           | Deeper analysis                           |
| Random amino acid chains        | Generate random polypeptides                              | Protein origins                           |
| Pattern emergence tracking      | Detect specific sequences in chains                       | Evolutionary order                        |
| Mutation/combination mechanisms | Simulate evolution by altering/recombining chains         | Diversity, adaptation                     |

---

These features together provide a comprehensive foundation for simulating the chemical and evolutionary processes relevant to abiogenesis. 