package com.abiogenesis.gui;

import com.abiogenesis.model.Molecule;
import com.abiogenesis.simulation.PrimordialSoup;
import javafx.application.Platform;
import javafx.scene.Scene;
import javafx.scene.canvas.Canvas;
import javafx.scene.canvas.GraphicsContext;
import javafx.scene.layout.BorderPane;
import javafx.scene.paint.Color;
import javafx.stage.Stage;

public class SimulationView {
    private final PrimordialSoup soup;
    private final Canvas canvas;
    private final GraphicsContext gc;
    private boolean isRunning = false;
    private static final int FPS = 60; // Increased FPS for smoother animation
    private static final double BASE_MOLECULE_SIZE = 6.0;

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
        BorderPane root = new BorderPane(canvas);
        Scene scene = new Scene(root);
        stage.setTitle("Abiogenesis Simulation");
        stage.setScene(scene);
        stage.show();

        // Start simulation loop
        startSimulation();
    }

    private void startSimulation() {
        isRunning = true;
        new Thread(() -> {
            while (isRunning) {
                soup.simulateStep();
                Platform.runLater(this::draw);
                try {
                    Thread.sleep(1000 / FPS); // 60 FPS
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }).start();
    }

    private void draw() {
        // Clear canvas with a dark background
        gc.setFill(Color.rgb(10, 10, 20)); // Dark blue-black background
        gc.fillRect(0, 0, canvas.getWidth(), canvas.getHeight());

        // Draw molecules with glow effect
        for (Molecule molecule : soup.getMolecules()) {
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