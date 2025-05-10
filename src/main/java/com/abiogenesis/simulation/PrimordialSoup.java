package com.abiogenesis.simulation;

import com.abiogenesis.model.Molecule;
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

    public PrimordialSoup(int width, int height, double temperature, double pH) {
        this.width = width;
        this.height = height;
        this.temperature = temperature;
        this.pH = pH;
        this.molecules = new ArrayList<>();
        this.random = new Random();
    }

    public void addMolecule(Molecule molecule) {
        if (isValidPosition(molecule.getPosition())) {
            molecules.add(molecule);
        }
    }

    public void simulateStep() {
        // Move molecules randomly
        for (Molecule molecule : molecules) {
            moveRandomly(molecule);
        }

        // Simulate molecular interactions
        for (int i = 0; i < molecules.size(); i++) {
            for (int j = i + 1; j < molecules.size(); j++) {
                Molecule m1 = molecules.get(i);
                Molecule m2 = molecules.get(j);
                
                // Check for potential reactions based on proximity and energy
                if (shouldReact(m1, m2)) {
                    // Simple reaction - combine molecules if conditions are right
                    if (m1.getName().equals("H2O") && m2.getName().equals("CH4")) {
                        // Remove reactants
                        molecules.remove(m2);
                        molecules.remove(m1);
                        
                        // Create new molecule as product
                        Molecule product = new Molecule("CH3OH", m1.getEnergy() + m2.getEnergy() - 0.5);
                        product.setPosition(m1.getPosition()); // Product forms at location of first reactant
                        molecules.add(product);
                        
                        // Break inner loop since m1 was removed
                        break;
                    }
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

    private boolean shouldReact(Molecule m1, Molecule m2) {
        double distance = m1.getPosition().distanceTo(m2.getPosition());
        double energyThreshold = 1.0; // Arbitrary threshold for now
        
        return distance < energyThreshold && 
               (m1.getEnergy() + m2.getEnergy()) > energyThreshold;
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
} 