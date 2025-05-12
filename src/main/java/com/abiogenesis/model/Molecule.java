package com.abiogenesis.model;

import java.util.ArrayList;
import java.util.List;

/**
 * Represents a generic chemical molecule in the simulation.
 * Each molecule has a name, an energy value, a list of atoms, and a position in space.
 * This class provides the basic structure for all molecules, including amino acid chains.
 */
public class Molecule {
    private String name;
    private double energy;
    private List<Atom> atoms;
    private Position position;

    /**
     * Creates a new molecule with the given name and energy.
     * @param name The name of the molecule (e.g., "H2O", "CH4").
     * @param energy The energy value of the molecule.
     */
    public Molecule(String name, double energy) {
        this.name = name;
        this.energy = energy;
        this.atoms = new ArrayList<>();
        this.position = new Position(0, 0);
    }

    /**
     * Returns the name of the molecule.
     * @return The molecule's name.
     */
    public String getName() {
        return name;
    }

    /**
     * Returns the energy value of the molecule.
     * @return The energy value.
     */
    public double getEnergy() {
        return energy;
    }

    /**
     * Sets the energy value of the molecule.
     * @param energy The new energy value.
     */
    public void setEnergy(double energy) {
        this.energy = energy;
    }

    /**
     * Returns the list of atoms that make up this molecule.
     * @return The list of atoms.
     */
    public List<Atom> getAtoms() {
        return atoms;
    }

    /**
     * Adds an atom to this molecule.
     * @param atom The atom to add.
     */
    public void addAtom(Atom atom) {
        atoms.add(atom);
    }

    /**
     * Returns the position of this molecule in the simulation space.
     * @return The position object.
     */
    public Position getPosition() {
        return position;
    }

    /**
     * Sets the position of this molecule in the simulation space.
     * @param position The new position.
     */
    public void setPosition(Position position) {
        this.position = position;
    }
} 