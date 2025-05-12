package com.abiogenesis.model;

/**
 * Represents a chain of amino acids (a simple polypeptide) in the simulation.
 * Each chain has a sequence of amino acids, an energy value, and atomic composition.
 * Provides methods for random generation, mutation, combination, and degradation (not currently used).
 */
public class AminoAcidMolecule extends Molecule {
    private String sequence;
    // The 20 standard amino acids, represented by their single-letter codes
    private static final char[] AMINO_ACIDS = {
        'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I',
        'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y', 'V'
    };
    // Controls how likely a chain is to degrade (not used in current phase)
    private static final double DEGRADATION_RATE = 0.05; // Reduced from 0.1 to make degradation slower
    // Minimum energy required for a chain to participate in reactions (not used in current phase)
    private static final double REACTION_THRESHOLD = 0.7; // Energy threshold for reactions

    /**
     * Creates a new amino acid chain with the given sequence and energy.
     * @param sequence The sequence of amino acids (single-letter codes).
     * @param energy The energy value of the chain.
     */
    public AminoAcidMolecule(String sequence, double energy) {
        super("AA" + sequence.length(), energy);
        this.sequence = sequence;
        // For each amino acid in the chain, add a simple atomic backbone (C, N, O, H)
        for (int i = 0; i < sequence.length(); i++) {
            addAtom(new Atom("C", 6, 12.011)); // Carbon backbone
            addAtom(new Atom("N", 7, 14.007)); // Nitrogen
            addAtom(new Atom("O", 8, 15.999)); // Oxygen
            addAtom(new Atom("H", 1, 1.008));  // Hydrogen
        }
    }

    /**
     * Returns the amino acid sequence of this chain.
     * @return The sequence as a string of single-letter codes.
     */
    public String getSequence() {
        return sequence;
    }

    /**
     * Generates a random amino acid chain of length between minLength and maxLength (inclusive).
     * @param minLength The minimum length of the chain.
     * @param maxLength The maximum length of the chain.
     * @return A new AminoAcidMolecule with a random sequence.
     */
    public static AminoAcidMolecule generateRandom(int minLength, int maxLength) {
        int length = minLength + (int)(Math.random() * (maxLength - minLength + 1));
        StringBuilder sb = new StringBuilder(length);
        for (int i = 0; i < length; i++) {
            char aa = AMINO_ACIDS[(int)(Math.random() * AMINO_ACIDS.length)];
            sb.append(aa);
        }
        return new AminoAcidMolecule(sb.toString(), 1.0 + (length * 0.1));
    }

    /**
     * Combines two amino acid chains into one longer chain (simple concatenation).
     * Energy is summed, minus a small cost for the reaction.
     * @param m1 The first chain.
     * @param m2 The second chain.
     * @return A new AminoAcidMolecule representing the combined chain.
     */
    public static AminoAcidMolecule combine(AminoAcidMolecule m1, AminoAcidMolecule m2) {
        String newSequence = m1.getSequence() + m2.getSequence();
        double newEnergy = m1.getEnergy() + m2.getEnergy() - 0.5; // Energy cost of combination
        return new AminoAcidMolecule(newSequence, newEnergy);
    }

    /**
     * Returns a new chain representing a degraded version of this one (shorter sequence).
     * Not used in the current phase.
     * @return A new AminoAcidMolecule with a shorter sequence, or null if already length 1.
     */
    public AminoAcidMolecule degrade() {
        if (sequence.length() <= 1) return null;
        // Randomly choose a position to split the chain
        int splitPoint = (int)(Math.random() * (sequence.length() - 1)) + 1;
        String newSequence = sequence.substring(0, splitPoint);
        double newEnergy = getEnergy() * 0.6; // Energy loss during degradation
        return new AminoAcidMolecule(newSequence, newEnergy);
    }

    /**
     * Checks if this chain and another can react (based on energy).
     * Not used in the current phase.
     * @param other The other amino acid chain.
     * @return True if both have enough energy, false otherwise.
     */
    public boolean canReactWith(AminoAcidMolecule other) {
        return getEnergy() > REACTION_THRESHOLD && other.getEnergy() > REACTION_THRESHOLD;
    }

    /**
     * Returns the probability that this chain will degrade (longer chains degrade more easily).
     * Not used in the current phase.
     * @return The degradation probability.
     */
    public double getDegradationProbability() {
        return DEGRADATION_RATE * (1.0 + (sequence.length() * 0.05));
    }

    /**
     * Returns a new chain with a single random amino acid substituted for another.
     * @return The mutated chain.
     */
    public AminoAcidMolecule mutateSubstitution() {
        if (sequence.length() == 0) return this;
        int pos = (int)(Math.random() * sequence.length());
        char[] chars = sequence.toCharArray();
        chars[pos] = AMINO_ACIDS[(int)(Math.random() * AMINO_ACIDS.length)];
        return new AminoAcidMolecule(new String(chars), getEnergy());
    }

    /**
     * Returns a new chain with a random amino acid inserted at a random position.
     * @return The mutated chain.
     */
    public AminoAcidMolecule mutateInsertion() {
        int pos = (int)(Math.random() * (sequence.length() + 1));
        char insert = AMINO_ACIDS[(int)(Math.random() * AMINO_ACIDS.length)];
        String newSeq = sequence.substring(0, pos) + insert + sequence.substring(pos);
        return new AminoAcidMolecule(newSeq, getEnergy());
    }

    /**
     * Returns a new chain with a single amino acid deleted (if length > 1).
     * @return The mutated chain.
     */
    public AminoAcidMolecule mutateDeletion() {
        if (sequence.length() <= 1) return this;
        int pos = (int)(Math.random() * sequence.length());
        String newSeq = sequence.substring(0, pos) + sequence.substring(pos + 1);
        return new AminoAcidMolecule(newSeq, getEnergy());
    }

    /**
     * Returns a new chain created by recombining (crossover) with another chain.
     * The new chain is a mix of the two parent sequences.
     * @param other The other parent chain.
     * @return The recombined chain.
     */
    public AminoAcidMolecule crossover(AminoAcidMolecule other) {
        if (sequence.length() == 0 || other.sequence.length() == 0) return this;
        int pos1 = (int)(Math.random() * sequence.length());
        int pos2 = (int)(Math.random() * other.sequence.length());
        String newSeq = sequence.substring(0, pos1) + other.sequence.substring(pos2);
        return new AminoAcidMolecule(newSeq, (getEnergy() + other.getEnergy()) / 2.0);
    }
} 