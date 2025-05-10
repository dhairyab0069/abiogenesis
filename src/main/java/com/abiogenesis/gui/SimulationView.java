package com.abiogenesis.gui;

import com.abiogenesis.model.Molecule;
import com.abiogenesis.simulation.PrimordialSoup;
import javafx.application.Platform;
import javafx.scene.Scene;
import javafx.scene.canvas.Canvas;
import javafx.scene.canvas.GraphicsContext;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.scene.control.Tooltip;
import javafx.stage.Stage;
import java.util.HashMap;
import java.util.Map;
import java.util.List;
import java.util.ArrayList;

public class SimulationView {
    private final PrimordialSoup soup;
    private final Canvas canvas;
    private final GraphicsContext gc;
    private boolean isRunning = false;
    private static final int FPS = 60; // Increased FPS for smoother animation
    private static final double BASE_MOLECULE_SIZE = 6.0;
    
    // Debug statistics
    private long lastFrameTime = 0;
    private int frameCount = 0;
    private long lastStatsTime = 0;
    private Map<String, Integer> moleculeCounts = new HashMap<>();
    private Tooltip moleculeTooltip = new Tooltip();

    public SimulationView(PrimordialSoup soup) {
        this.soup = soup;
        this.canvas = new Canvas(soup.getWidth(), soup.getHeight());
        this.gc = canvas.getGraphicsContext2D();
        setupCanvas();
    }

    private void setupCanvas() {
        gc.setLineWidth(1.0);
        gc.setStroke(Color.WHITE);
    }

    public void show(Stage stage) {
        Pane root = new Pane(canvas);
        Scene scene = new Scene(root);
        stage.setTitle("Abiogenesis Simulation");
        stage.setScene(scene);
        stage.show();

        // Add mouse move listener for tooltips
        canvas.setOnMouseMoved(event -> {
            double x = event.getX();
            double y = event.getY();
            boolean found = false;
            for (Molecule molecule : new ArrayList<>(soup.getMolecules())) {
                double moleculeX = molecule.getPosition().getX();
                double moleculeY = molecule.getPosition().getY();
                double size = BASE_MOLECULE_SIZE + (molecule.getEnergy() * 2);
                if (Math.hypot(x - moleculeX, y - moleculeY) < size) {
                    StringBuilder tooltipText = new StringBuilder();
                    tooltipText.append(String.format("Molecule: %s\n", molecule.getName()));
                    tooltipText.append(String.format("Energy: %.2f\n", molecule.getEnergy()));
                    tooltipText.append(String.format("Position: (%.1f, %.1f)\n", 
                        molecule.getPosition().getX(), molecule.getPosition().getY()));
                    tooltipText.append("\nAtomic Composition:\n");
                    Map<String, Integer> atomCounts = new HashMap<>();
                    molecule.getAtoms().forEach(atom -> 
                        atomCounts.merge(atom.getElement(), 1, Integer::sum));
                    atomCounts.forEach((element, count) -> 
                        tooltipText.append(String.format("  %s: %d\n", element, count)));
                    tooltipText.append("\nStructure:\n");
                    switch (molecule.getName()) {
                        case "H2O":
                            tooltipText.append("  H-O-H (Bent)\n");
                            break;
                        case "CH4":
                            tooltipText.append("  Tetrahedral\n");
                            break;
                        case "CH3OH":
                            tooltipText.append("  H3C-OH (Methyl Alcohol)\n");
                            break;
                    }
                    moleculeTooltip.setText(tooltipText.toString());
                    moleculeTooltip.setStyle("-fx-font-size: 12px;");
                    moleculeTooltip.show(canvas, event.getScreenX() + 10, event.getScreenY() + 10);
                    found = true;
                    break;
                }
            }
            if (!found) {
                moleculeTooltip.hide();
            }
        });

        // Hide tooltip when mouse exits canvas
        canvas.setOnMouseExited(event -> moleculeTooltip.hide());

        // Start simulation loop
        startSimulation();
    }

    private void startSimulation() {
        isRunning = true;
        lastFrameTime = System.nanoTime();
        lastStatsTime = System.nanoTime();
        
        new Thread(() -> {
            while (isRunning) {
                long currentTime = System.nanoTime();
                long frameTime = currentTime - lastFrameTime;
                lastFrameTime = currentTime;
                
                soup.simulateStep();
                Platform.runLater(this::draw);
                
                // Update debug statistics
                updateDebugStats();
                
                try {
                    Thread.sleep(Math.max(0, (1000 / FPS) - (frameTime / 1_000_000)));
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }).start();
    }

    private void updateDebugStats() {
        frameCount++;
        long currentTime = System.nanoTime();
        
        // Update molecule counts
        moleculeCounts.clear();
        for (Molecule molecule : soup.getMolecules()) {
            moleculeCounts.merge(molecule.getName(), 1, Integer::sum);
        }
        
        // Print statistics every second
        if (currentTime - lastStatsTime >= 1_000_000_000) { // 1 second in nanoseconds
            double fps = frameCount * 1_000_000_000.0 / (currentTime - lastStatsTime);
            System.out.println("\n=== Simulation Statistics ===");
            System.out.printf("FPS: %.2f%n", fps);
            System.out.println("Molecule Counts:");
            moleculeCounts.forEach((name, count) -> 
                System.out.printf("  %s: %d%n", name, count));
            System.out.printf("Total Molecules: %d%n", soup.getMolecules().size());
            System.out.printf("Temperature: %.1fK%n", soup.getTemperature());
            System.out.printf("pH: %.1f%n", soup.getPH());
            System.out.printf("Total Reactions: %d%n", soup.getTotalReactions());
            System.out.printf("Reactions This Step: %d%n", soup.getReactionsThisStep());
            System.out.println("========================\n");
            
            // Reset counters
            frameCount = 0;
            lastStatsTime = currentTime;
        }
    }

    private void draw() {
        // Clear canvas with a dark background
        gc.setFill(Color.rgb(10, 10, 20)); // Dark blue-black background
        gc.fillRect(0, 0, canvas.getWidth(), canvas.getHeight());

        // Create a thread-safe copy of molecules for drawing
        List<Molecule> moleculesCopy = new ArrayList<>(soup.getMolecules());

        // Draw molecules with glow effect
        for (Molecule molecule : moleculesCopy) {
            double x = molecule.getPosition().getX();
            double y = molecule.getPosition().getY();
            double size = BASE_MOLECULE_SIZE + (molecule.getEnergy() * 2);
            
            // Determine molecule color
            Color color;
            switch (molecule.getName()) {
                case "H2O":
                    color = Color.AQUA;
                    break;
                case "CH4":
                    color = Color.LIGHTGREEN;
                    break;
                case "CH3OH":
                    color = Color.PURPLE;
                    break;
                default:
                    color = Color.WHITE;
            }

            // Draw glow effect
            gc.setFill(color.deriveColor(1, 1, 1, 0.3));
            gc.fillOval(x - size * 0.7, y - size * 0.7, size * 1.4, size * 1.4);
            
            // Draw molecule
            gc.setFill(color);
            gc.fillOval(x - size * 0.5, y - size * 0.5, size, size);
        }
    }

    public void stop() {
        isRunning = false;
    }
} 