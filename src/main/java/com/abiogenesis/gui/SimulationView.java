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

    public SimulationView(PrimordialSoup soup) {
        this.soup = soup;
        this.canvas = new Canvas(soup.getWidth(), soup.getHeight());
        this.gc = canvas.getGraphicsContext2D();
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
                    Thread.sleep(100); // 10 FPS
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }).start();
    }

    private void draw() {
        // Clear canvas
        gc.setFill(Color.BLACK);
        gc.fillRect(0, 0, canvas.getWidth(), canvas.getHeight());

        // Draw molecules
        for (Molecule molecule : soup.getMolecules()) {
            double x = molecule.getPosition().getX();
            double y = molecule.getPosition().getY();
            
            // Color based on molecule type
            switch (molecule.getName()) {
                case "H2O":
                    gc.setFill(Color.BLUE);
                    break;
                case "CH4":
                    gc.setFill(Color.GREEN);
                    break;
                default:
                    gc.setFill(Color.WHITE);
            }
            
            gc.fillOval(x - 5, y - 5, 10, 10);
        }
    }

    public void stop() {
        isRunning = false;
    }
} 