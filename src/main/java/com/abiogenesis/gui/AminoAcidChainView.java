package com.abiogenesis.gui;

import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;

public class AminoAcidChainView {
    public void show(Stage stage) {
        StackPane root = new StackPane(new Label("Amino Acid Chain Simulation (Coming Soon)"));
        root.setStyle("-fx-background-color: #181c1f;");
        Scene scene = new Scene(root, 800, 600);
        stage.setTitle("Amino Acid Chain Simulation");
        stage.setScene(scene);
        stage.show();
    }
} 