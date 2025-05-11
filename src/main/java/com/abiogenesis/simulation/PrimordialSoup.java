package com.abiogenesis.simulation;

import com.abiogenesis.model.Molecule;
import com.abiogenesis.model.AminoAcidMolecule;

import com.abiogenesis.model.Position;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

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
    
    // Generation parameters
    private static final double AMINO_ACID_GENERATION_RATE = 0.005; // Slower generation
    private static final int MAX_AMINO_ACIDS = 50; // Temporary threshold
    private static final String TARGET_PATTERN = "METHINKS";
    private static final double MUTATION_RATE = 0.01; // 1% chance per step per chain
    private static final double COMBINATION_RATE = 0.01; // 1% chance per step per pair

    public PrimordialSoup(int width, int height, double temperature, double pH) {
        this.width = width;
        this.height = height;
        this.temperature = temperature;
        this.pH = pH;
        this.molecules = new ArrayList<>();
        this.random = new Random();
        
        // Initialize with some amino acid molecules
        for (int i = 0; i < INITIAL_AMINO_ACIDS; i++) {
            AminoAcidMolecule aa = AminoAcidMolecule.generateRandom(1, 3);
            aa.getPosition().setX(random.nextDouble() * width);
            aa.getPosition().setY(random.nextDouble() * height);
            molecules.add(aa);
        }
    }

    private void generateAminoAcids() {
        // Count current amino acids
        int aminoAcidCount = 0;
        for (Molecule m : molecules) {
            if (m instanceof AminoAcidMolecule) {
                aminoAcidCount++;
            }
        }
        // Generate new amino acids if under threshold
        if (aminoAcidCount < MAX_AMINO_ACIDS && random.nextDouble() < AMINO_ACID_GENERATION_RATE) {
            AminoAcidMolecule newAA = AminoAcidMolecule.generateRandom(1, 3);
            newAA.getPosition().setX(random.nextDouble() * width);
            newAA.getPosition().setY(random.nextDouble() * height);
            molecules.add(newAA);
        }
    }

    public void addMolecule(Molecule molecule) {
        if (isValidPosition(molecule.getPosition())) {
            molecules.add(molecule);
        }
    }

    public void simulateStep() {
        reactionsThisStep = 0; // Reset reaction counter for this step
        
        // Only generate amino acids
        generateAminoAcids();
        
        // --- Mutation ---
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
        
        // Move molecules randomly
        for (Molecule molecule : molecules) {
            moveRandomly(molecule);
        }
        // --- Pattern Emergence Tracking ---
        for (Molecule molecule : molecules) {
            if (molecule instanceof AminoAcidMolecule) {
                AminoAcidMolecule aa = (AminoAcidMolecule) molecule;
                if (aa.getSequence().contains(TARGET_PATTERN)) {
                    System.out.println("Pattern '" + TARGET_PATTERN + "' found in chain: " + aa.getSequence());
                }
            }
        }
    }

    private void moveRandomly(Molecule molecule) {
        Position pos = molecule.getPosition();
        
        // Calculate random movement based on temperature
        double movementScale = MOVEMENT_SPEED * (temperature / 300.0); // Scale movement with temperature
        double dx = (random.nextDouble() - 0.5) * movementScale;
        double dy = (random.nextDouble() - 0.5) * movementScale;
        
        // Update position with bounds checking
        double newX = Math.max(0, Math.min(width, pos.getX() + dx));
        double newY = Math.max(0, Math.min(height, pos.getY() + dy));
        
        pos.setX(newX);
        pos.setY(newY);
    }

    private boolean isValidPosition(Position pos) {
        return pos.getX() >= 0 && pos.getX() < width &&
               pos.getY() >= 0 && pos.getY() < height;
    }

    public List<Molecule> getMolecules() {
        return molecules;
    }

    public double getTemperature() {
        return temperature;
    }

    public void setTemperature(double temperature) {
        this.temperature = temperature;
    }

    public double getPH() {
        return pH;
    }

    public void setPH(double pH) {
        this.pH = pH;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public int getTotalReactions() {
        return totalReactions;
    }

    public int getReactionsThisStep() {
        return reactionsThisStep;
    }

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