package com.abiogenesis.gui;

import javafx.application.Application;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import com.abiogenesis.simulation.PrimordialSoup;
import com.abiogenesis.model.Molecule;
import com.abiogenesis.model.Atom;

public class Launcher extends Application {
    @Override
    public void start(Stage primaryStage) {
        Button moleculeSimBtn = new Button("Molecule Simulation");
        Button aminoAcidSimBtn = new Button("Amino Acid Chain Simulation");

        moleculeSimBtn.setOnAction(e -> {
            // Launch Molecule Simulation with initialized molecules
            PrimordialSoup soup = new PrimordialSoup(800, 600, 300.0, 7.0);
            for (int i = 0; i < 50; i++) {
                Molecule water = new Molecule("H2O", 1.0);
                water.addAtom(new Atom("H", 1, 1.008));
                water.addAtom(new Atom("H", 1, 1.008));
                water.addAtom(new Atom("O", 8, 15.999));
                water.getPosition().setX(Math.random() * soup.getWidth());
                water.getPosition().setY(Math.random() * soup.getHeight());
                soup.addMolecule(water);

                Molecule methane = new Molecule("CH4", 1.5);
                methane.addAtom(new Atom("C", 6, 12.011));
                methane.addAtom(new Atom("H", 1, 1.008));
                methane.addAtom(new Atom("H", 1, 1.008));
                methane.addAtom(new Atom("H", 1, 1.008));
                methane.addAtom(new Atom("H", 1, 1.008));
                methane.getPosition().setX(Math.random() * soup.getWidth());
                methane.getPosition().setY(Math.random() * soup.getHeight());
                soup.addMolecule(methane);
            }
            SimulationView simView = new SimulationView(soup);
            simView.show(primaryStage);
            // Add a way to return to menu (to be implemented in SimulationView)
        });

        aminoAcidSimBtn.setOnAction(e -> {
            // Launch Amino Acid Chain Simulation (stub)
            AminoAcidChainView chainView = new AminoAcidChainView();
            chainView.show(primaryStage);
            // Add a way to return to menu (to be implemented in AminoAcidChainView)
        });

        VBox root = new VBox(20, moleculeSimBtn, aminoAcidSimBtn);
        root.setStyle("-fx-padding: 40; -fx-alignment: center; -fx-background-color: #181c1f;");
        Scene scene = new Scene(root, 400, 300);
        primaryStage.setTitle("Abiogenesis Simulation Launcher");
        primaryStage.setScene(scene);
        primaryStage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
} 