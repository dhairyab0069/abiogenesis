package com.abiogenesis;

import com.abiogenesis.gui.SimulationView;
import com.abiogenesis.model.Atom;
import com.abiogenesis.model.Molecule;
import com.abiogenesis.simulation.PrimordialSoup;
import javafx.application.Application;
import javafx.stage.Stage;

public class Main extends Application {
    @Override
    public void start(Stage primaryStage) {
        // Create a primordial soup environment
        PrimordialSoup soup = new PrimordialSoup(800, 600, 300.0, 7.0); // 300K, neutral pH

        // Add some basic molecules with random positions
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

        // Create and show the simulation view
        SimulationView view = new SimulationView(soup);
        view.show(primaryStage);
    }

    public static void main(String[] args) {
        launch(args);
    }
} 