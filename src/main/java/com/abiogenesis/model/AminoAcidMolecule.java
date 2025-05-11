package com.abiogenesis.model;

public class AminoAcidMolecule extends Molecule {
    private String sequence;
    private static final char[] AMINO_ACIDS = {
        'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I',
        'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y', 'V'
    };
    private static final double DEGRADATION_RATE = 0.05; // Reduced from 0.1 to make degradation slower
    private static final double REACTION_THRESHOLD = 0.7; // Energy threshold for reactions

    public AminoAcidMolecule(String sequence, double energy) {
        super("AA" + sequence.length(), energy);
        this.sequence = sequence;
        // Add atoms based on sequence length
        for (int i = 0; i < sequence.length(); i++) {
            addAtom(new Atom("C", 6, 12.011)); // Carbon backbone
            addAtom(new Atom("N", 7, 14.007)); // Nitrogen
            addAtom(new Atom("O", 8, 15.999)); // Oxygen
            addAtom(new Atom("H", 1, 1.008));  // Hydrogen
        }
    }

    public String getSequence() {
        return sequence;
    }

    public static AminoAcidMolecule generateRandom(int minLength, int maxLength) {
        int length = minLength + (int)(Math.random() * (maxLength - minLength + 1));
        StringBuilder sb = new StringBuilder(length);
        for (int i = 0; i < length; i++) {
            char aa = AMINO_ACIDS[(int)(Math.random() * AMINO_ACIDS.length)];
            sb.append(aa);
        }
        return new AminoAcidMolecule(sb.toString(), 1.0 + (length * 0.1));
    }

    public static AminoAcidMolecule combine(AminoAcidMolecule m1, AminoAcidMolecule m2) {
        String newSequence = m1.getSequence() + m2.getSequence();
        double newEnergy = m1.getEnergy() + m2.getEnergy() - 0.5; // Energy cost of combination
        return new AminoAcidMolecule(newSequence, newEnergy);
    }

    public AminoAcidMolecule degrade() {
        if (sequence.length() <= 1) return null;
        
        // Randomly choose a position to split
        int splitPoint = (int)(Math.random() * (sequence.length() - 1)) + 1;
        String newSequence = sequence.substring(0, splitPoint);
        double newEnergy = getEnergy() * 0.6; // Energy loss during degradation
        return new AminoAcidMolecule(newSequence, newEnergy);
    }

    public boolean canReactWith(AminoAcidMolecule other) {
        // Check if both molecules have sufficient energy
        return getEnergy() > REACTION_THRESHOLD && other.getEnergy() > REACTION_THRESHOLD;
    }

    public double getDegradationProbability() {
        // Longer chains are more likely to degrade
        return DEGRADATION_RATE * (1.0 + (sequence.length() * 0.05));
    }
} 