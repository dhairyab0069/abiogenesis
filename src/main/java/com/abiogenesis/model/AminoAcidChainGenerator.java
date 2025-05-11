package com.abiogenesis.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

public class AminoAcidChainGenerator {
    private static final char[] AMINO_ACIDS = {
        'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I',
        'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y', 'V'
    };
    private static final int MIN_LENGTH = 5;
    private static final int MAX_LENGTH = 20;
    private static final Random random = new Random();

    public static String generateRandomChain() {
        int length = MIN_LENGTH + random.nextInt(MAX_LENGTH - MIN_LENGTH + 1);
        StringBuilder sb = new StringBuilder(length);
        for (int i = 0; i < length; i++) {
            char aa = AMINO_ACIDS[random.nextInt(AMINO_ACIDS.length)];
            sb.append(aa);
        }
        return sb.toString();
    }

    public static List<String> generateRandomChains(int count) {
        List<String> chains = new ArrayList<>();
        for (int i = 0; i < count; i++) {
            chains.add(generateRandomChain());
        }
        return chains;
    }
} 