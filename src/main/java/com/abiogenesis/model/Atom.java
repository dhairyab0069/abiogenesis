package com.abiogenesis.model;

public class Atom {
    private String element;
    private int atomicNumber;
    private double atomicMass;

    public Atom(String element, int atomicNumber, double atomicMass) {
        this.element = element;
        this.atomicNumber = atomicNumber;
        this.atomicMass = atomicMass;
    }

    public String getElement() {
        return element;
    }

    public int getAtomicNumber() {
        return atomicNumber;
    }

    public double getAtomicMass() {
        return atomicMass;
    }
} 