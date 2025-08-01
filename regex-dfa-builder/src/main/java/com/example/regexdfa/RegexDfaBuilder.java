package com.example.regexdfa;

import dk.brics.automaton.Automaton;
import dk.brics.automaton.RegExp;
import dk.brics.automaton.State;
import dk.brics.automaton.Transition;

// import java.util.*;

/**
 * Main demo class for converting regex to DFA using dk.brics.automaton
 */
public class RegexDfaBuilder {
    
    public static void main(String[] args) {
        if (args.length == 0 || args.length > 2) {
            System.err.println("❗ Please provide a regular expression as a command-line argument.");
            System.err.println("👉 Usage: java RegexDfaDemo \"<regular expression>\"");
            return;
        } 

        String inputRegex = args[0];

        try {
            // System.out.println("Starting Regex to DFA conversion demo...");
            // System.out.println("Input regex: " + inputRegex);
            
            // Step 1: Parse regex string to automaton
            // System.out.println("\nStep 1: Parsing regex to automaton...");
            RegExp regExp = new RegExp(inputRegex);
            Automaton regexAutomaton = regExp.toAutomaton();
            // System.out.println("Regex automaton created with " + regexAutomaton.getNumberOfStates() + " states");
            
            // Step 2: Create Σ* automaton (0-255 characters)
            // System.out.println("\nStep 2: Creating Σ* automaton (0-255 characters)...");
            Automaton sigmaStarAutomaton = createSigmaStarAutomaton();
            // System.out.println("Σ* automaton created with " + sigmaStarAutomaton.getNumberOfStates() + " states");
            
            // Step 3: Intersect with Σ* automaton
            // System.out.println("\nStep 3: Intersecting regex automaton with Σ*...");
            Automaton intersectedAutomaton = regexAutomaton.intersection(sigmaStarAutomaton);
            // System.out.println("Intersected automaton has " + intersectedAutomaton.getNumberOfStates() + " states");
            
            // Step 4: Minimize the automaton
            // System.out.println("\nStep 4: Minimizing automaton...");
            intersectedAutomaton.minimize();
            // System.out.println("Minimized automaton has " + intersectedAutomaton.getNumberOfStates() + " states");
            
            // // Step 5 (maybe not needed?): Complete the DFA (add garbage state for missing transitions)
            // System.out.println("\nStep 5: Completing DFA (adding garbage state for missing transitions)...");
            // intersectedAutomaton = completeDfa(intersectedAutomaton);
            // System.out.println("Completed DFA has " + intersectedAutomaton.getNumberOfStates() + " states");

            // Step 6: Serialize to JSON
            // System.out.println("\nStep 6: Serializing to JSON...");
            String hash = sha256Hex(inputRegex).substring(0, 8);  // Use first 8 chars
            String outputFilename = "dfa-" + hash + ".json";
            AutomatonSerializer.serializeToJson(intersectedAutomaton, inputRegex, outputFilename);
            System.out.println(outputFilename); //Output filename so parent program can locate the output file
            
            // System.out.println("\nDemo completed successfully!");
            
        } catch (Exception e) {
            System.err.println("Error occurred: " + e.getMessage());
            e.printStackTrace();
        }
    }
    
    /**
     * Creates a Σ* automaton that accepts all strings over alphabet 0-255
     */
    private static Automaton createSigmaStarAutomaton() {
        Automaton sigmaStarAutomaton = new Automaton();
        State state = new State();
        state.setAccept(true); // Initial state is also accepting (accepts empty string)
        
        // Add self-loop for all characters 0-255
        state.addTransition(new Transition((char)0, (char)255, state));
        
        sigmaStarAutomaton.setInitialState(state);
        sigmaStarAutomaton.setDeterministic(true);
        
        return sigmaStarAutomaton;
    }

    private static String sha256Hex(String input) throws Exception {
        java.security.MessageDigest digest = java.security.MessageDigest.getInstance("SHA-256");
        byte[] hash = digest.digest(input.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        StringBuilder hex = new StringBuilder();
        for (byte b : hash) {
            hex.append(String.format("%02x", b));
        }
        return hex.toString();
    }
} 