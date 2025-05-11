package com.abiogenesis.gui;

import com.abiogenesis.model.AminoAcidChainGenerator;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import java.util.List;

public class AminoAcidChainView {
    private static final int CHAIN_COUNT = 20;
    private VBox chainBox;

    public void show(Stage stage) {
        chainBox = new VBox(5);
        chainBox.setStyle("-fx-padding: 20; -fx-background-color: #181c1f;");
        Label title = new Label("Amino Acid Chain Simulation");
        title.setStyle("-fx-font-size: 20px; -fx-text-fill: white; -fx-padding: 0 0 10 0;");
        Button regenerateBtn = new Button("Regenerate Chains");
        regenerateBtn.setOnAction(e -> displayChains());
        VBox root = new VBox(15, title, regenerateBtn, chainBox);
        root.setStyle("-fx-padding: 40; -fx-alignment: center; -fx-background-color: #181c1f;");
        displayChains();
        Scene scene = new Scene(root, 800, 600);
        stage.setTitle("Amino Acid Chain Simulation");
        stage.setScene(scene);
        stage.show();
    }

    private void displayChains() {
        chainBox.getChildren().clear();
        List<String> chains = AminoAcidChainGenerator.generateRandomChains(CHAIN_COUNT);
        for (String chain : chains) {
            Label label = new Label(chain);
            label.setStyle("-fx-font-size: 16px; -fx-text-fill: #b3e5fc; -fx-font-family: 'monospace';");
            chainBox.getChildren().add(label);
        }
    }
} 