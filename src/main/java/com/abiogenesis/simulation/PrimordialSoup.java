package com.abiogenesis.simulation;

import com.abiogenesis.model.Molecule;
import com.abiogenesis.model.AminoAcidMolecule;

import com.abiogenesis.model.Position;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * The PrimordialSoup class simulates a 2D environment where amino acid chains are generated, move randomly,
 * and undergo mutation and recombination. It models a simplified version of prebiotic chemistry and evolution.
 *
 * Features:
 * - Random generation of amino acid chains up to a maximum population.
 * - Temperature-dependent Brownian motion for all molecules.
 * - Mutation (substitution, insertion, deletion) and recombination (crossover) of chains.
 * - Pattern tracking for the emergence of specific sequences (e.g., "METHINKS").
 * - Population control to prevent runaway growth.
 */
public class PrimordialSoup {
    private List<Molecule> molecules;
    private double temperature;
    private double pH;
    private int width;
    private int height;
    private Random random;
    private static final double MOVEMENT_SPEED = 2.0;
    private int totalReactions = 0;
    private int reactionsThisStep = 0;
    private static final int INITIAL_AMINO_ACIDS = 20;
    
    // Controls how often new amino acid chains are generated (probability per step)
    private static final double AMINO_ACID_GENERATION_RATE = 0.005; // Slower generation
    // Maximum number of amino acid chains allowed in the simulation at any time
    private static final int MAX_AMINO_ACIDS = 50; // Temporary threshold
    // The sequence we want to track for spontaneous emergence
    private static final String TARGET_PATTERN = "METHINKS";
    // Probability that a chain will mutate each step
    private static final double MUTATION_RATE = 0.01; // 1% chance per step per chain
    // Probability that a pair of chains will recombine each step
    private static final double COMBINATION_RATE = 0.01; // 1% chance per step per pair

    /**
     * Creates a new PrimordialSoup simulation environment.
     * @param width The width of the simulation area.
     * @param height The height of the simulation area.
     * @param temperature The temperature of the environment (affects movement).
     * @param pH The pH of the environment (not currently used).
     */
    public PrimordialSoup(int width, int height, double temperature, double pH) {
        this.width = width;
        this.height = height;
        this.temperature = temperature;
        this.pH = pH;
        this.molecules = new ArrayList<>();
        this.random = new Random();
        
        // Start the simulation with a handful of random amino acid chains, scattered throughout the soup
        for (int i = 0; i < INITIAL_AMINO_ACIDS; i++) {
            AminoAcidMolecule aa = AminoAcidMolecule.generateRandom(1, 3);
            aa.getPosition().setX(random.nextDouble() * width);
            aa.getPosition().setY(random.nextDouble() * height);
            molecules.add(aa);
        }
    }

    /**
     * Attempts to generate a new amino acid chain, but only if the population is below the maximum allowed.
     * The new chain is placed at a random location in the soup.
     */
    private void generateAminoAcids() {
        int aminoAcidCount = 0;
        for (Molecule m : molecules) {
            if (m instanceof AminoAcidMolecule) {
                aminoAcidCount++;
            }
        }
        // Only add a new chain if we have room for it
        if (aminoAcidCount < MAX_AMINO_ACIDS && random.nextDouble() < AMINO_ACID_GENERATION_RATE) {
            AminoAcidMolecule newAA = AminoAcidMolecule.generateRandom(1, 3);
            newAA.getPosition().setX(random.nextDouble() * width);
            newAA.getPosition().setY(random.nextDouble() * height);
            molecules.add(newAA);
        }
    }

    /**
     * Adds a molecule to the simulation, as long as its position is valid (inside the soup).
     * @param molecule The molecule to add.
     */
    public void addMolecule(Molecule molecule) {
        if (isValidPosition(molecule.getPosition())) {
            molecules.add(molecule);
        }
    }

    /**
     * Advances the simulation by one step: generates new chains, mutates and recombines chains,
     * moves all molecules, and checks for the emergence of the target pattern.
     */
    public void simulateStep() {
        reactionsThisStep = 0; // Reset reaction counter for this step
        
        // Try to add a new amino acid chain if there's room
        generateAminoAcids();
        
        // --- Mutation ---
        // Each chain has a small chance to mutate (substitution, insertion, or deletion)
        List<Molecule> mutatedMolecules = new ArrayList<>();
        for (Molecule molecule : molecules) {
            if (molecule instanceof AminoAcidMolecule) {
                AminoAcidMolecule aa = (AminoAcidMolecule) molecule;
                double r = random.nextDouble();
                if (r < MUTATION_RATE) {
                    int mutType = random.nextInt(3);
                    AminoAcidMolecule mutated;
                    if (mutType == 0) mutated = aa.mutateSubstitution();
                    else if (mutType == 1) mutated = aa.mutateInsertion();
                    else mutated = aa.mutateDeletion();
                    mutated.getPosition().setX(aa.getPosition().getX());
                    mutated.getPosition().setY(aa.getPosition().getY());
                    mutatedMolecules.add(mutated);
                } else {
                    mutatedMolecules.add(aa);
                }
            } else {
                mutatedMolecules.add(molecule);
            }
        }
        molecules = mutatedMolecules;
        
        // --- Combination (crossover) ---
        // Randomly select up to 10 pairs of chains to recombine, but only if under the population cap
        List<Molecule> currentMolecules = new ArrayList<>(molecules);
        int combinations = 0;
        int maxCombinations = 10;
        while (combinations < maxCombinations && molecules.size() < MAX_AMINO_ACIDS) {
            if (currentMolecules.size() < 2) break;
            int i = random.nextInt(currentMolecules.size());
            int j = random.nextInt(currentMolecules.size());
            if (i == j) continue;
            Molecule m1 = currentMolecules.get(i);
            Molecule m2 = currentMolecules.get(j);
            if (m1 instanceof AminoAcidMolecule && m2 instanceof AminoAcidMolecule) {
                if (random.nextDouble() < COMBINATION_RATE) {
                    AminoAcidMolecule aa1 = (AminoAcidMolecule) m1;
                    AminoAcidMolecule aa2 = (AminoAcidMolecule) m2;
                    AminoAcidMolecule child = aa1.crossover(aa2);
                    child.getPosition().setX((aa1.getPosition().getX() + aa2.getPosition().getX()) / 2.0);
                    child.getPosition().setY((aa1.getPosition().getY() + aa2.getPosition().getY()) / 2.0);
                    molecules.add(child);
                    combinations++;
                }
            }
        }
        
        // Move every molecule in a random direction, with the amount of movement depending on temperature
        for (Molecule molecule : molecules) {
            moveRandomly(molecule);
        }
        // --- Pattern Emergence Tracking ---
        // After all changes, check if any chain contains the target pattern
        for (Molecule molecule : molecules) {
            if (molecule instanceof AminoAcidMolecule) {
                AminoAcidMolecule aa = (AminoAcidMolecule) molecule;
                if (aa.getSequence().contains(TARGET_PATTERN)) {
                    System.out.println("Pattern '" + TARGET_PATTERN + "' found in chain: " + aa.getSequence());
                }
            }
        }
    }

    /**
     * Moves a molecule in a random direction, with the distance scaled by the current temperature.
     * Ensures the molecule stays within the simulation boundaries.
     * @param molecule The molecule to move.
     */
    private void moveRandomly(Molecule molecule) {
        Position pos = molecule.getPosition();
        // The higher the temperature, the more energetic the movement
        double movementScale = MOVEMENT_SPEED * (temperature / 300.0);
        double dx = (random.nextDouble() - 0.5) * movementScale;
        double dy = (random.nextDouble() - 0.5) * movementScale;
        // Keep the molecule inside the soup
        double newX = Math.max(0, Math.min(width, pos.getX() + dx));
        double newY = Math.max(0, Math.min(height, pos.getY() + dy));
        pos.setX(newX);
        pos.setY(newY);
    }

    /**
     * Checks if a position is inside the simulation area.
     * @param pos The position to check.
     * @return True if the position is valid, false otherwise.
     */
    private boolean isValidPosition(Position pos) {
        return pos.getX() >= 0 && pos.getX() < width &&
               pos.getY() >= 0 && pos.getY() < height;
    }

    /**
     * Returns the list of all molecules currently in the simulation.
     * @return The list of molecules.
     */
    public List<Molecule> getMolecules() {
        return molecules;
    }

    /**
     * Gets the current temperature of the soup.
     * @return The temperature.
     */
    public double getTemperature() {
        return temperature;
    }

    /**
     * Sets the temperature of the soup.
     * @param temperature The new temperature.
     */
    public void setTemperature(double temperature) {
        this.temperature = temperature;
    }

    /**
     * Gets the current pH of the soup.
     * @return The pH value.
     */
    public double getPH() {
        return pH;
    }

    /**
     * Sets the pH of the soup.
     * @param pH The new pH value.
     */
    public void setPH(double pH) {
        this.pH = pH;
    }

    /**
     * Gets the width of the simulation area.
     * @return The width.
     */
    public int getWidth() {
        return width;
    }

    /**
     * Gets the height of the simulation area.
     * @return The height.
     */
    public int getHeight() {
        return height;
    }

    /**
     * Gets the total number of reactions that have occurred (not used in this phase).
     * @return The total reaction count.
     */
    public int getTotalReactions() {
        return totalReactions;
    }

    /**
     * Gets the number of reactions that occurred in the last step (not used in this phase).
     * @return The reaction count for the last step.
     */
    public int getReactionsThisStep() {
        return reactionsThisStep;
    }

    /**
     * Sets the width of the simulation area and rescales all molecule positions accordingly.
     * @param width The new width.
     */
    public void setWidth(int width) {
        if (this.width > 0 && width > 0 && this.width != width) {
            double scale = (double) width / this.width;
            for (Molecule molecule : molecules) {
                double newX = molecule.getPosition().getX() * scale;
                molecule.getPosition().setX(newX);
            }
        }
        this.width = width;
    }

    /**
     * Sets the height of the simulation area and rescales all molecule positions accordingly.
     * @param height The new height.
     */
    public void setHeight(int height) {
        if (this.height > 0 && height > 0 && this.height != height) {
            double scale = (double) height / this.height;
            for (Molecule molecule : molecules) {
                double newY = molecule.getPosition().getY() * scale;
                molecule.getPosition().setY(newY);
            }
        }
        this.height = height;
    }
} 