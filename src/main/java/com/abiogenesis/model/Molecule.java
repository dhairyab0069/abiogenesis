package com.abiogenesis.model;

import java.util.ArrayList;
import java.util.List;

public class Molecule {
    private String name;
    private double energy;
    private List<Atom> atoms;
    private Position position;

    public Molecule(String name, double energy) {
        this.name = name;
        this.energy = energy;
        this.atoms = new ArrayList<>();
        this.position = new Position(0, 0);
    }

    public void addAtom(Atom atom) {
        atoms.add(atom);
    }

    public double getEnergy() {
        return energy;
    }

    public void setEnergy(double energy) {
        this.energy = energy;
    }

    public Position getPosition() {
        return position;
    }

    public void setPosition(Position position) {
        this.position = position;
    }

    public String getName() {
        return name;
    }

    public List<Atom> getAtoms() {
        return atoms;
    }
} 