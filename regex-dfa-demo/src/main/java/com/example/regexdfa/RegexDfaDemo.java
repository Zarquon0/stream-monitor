package com.example.regexdfa;

import dk.brics.automaton.Automaton;
import dk.brics.automaton.RegExp;
import dk.brics.automaton.State;
import dk.brics.automaton.Transition;

import java.io.IOException;
import java.util.*;

/**
 * Main demo class for converting regex to DFA using dk.brics.automaton
 */
public class RegexDfaDemo {
    
    private static final String SAMPLE_REGEX = "[0-9]+";
    private static final String OUTPUT_FILENAME = "automaton.json";
    
    public static void main(String[] args) {
        try {
            System.out.println("Starting Regex to DFA conversion demo...");
            System.out.println("Input regex: " + SAMPLE_REGEX);
            
            // Step 1: Parse regex string to automaton
            System.out.println("\nStep 1: Parsing regex to automaton...");
            RegExp regExp = new RegExp(SAMPLE_REGEX);
            Automaton regexAutomaton = regExp.toAutomaton();
            System.out.println("Regex automaton created with " + regexAutomaton.getNumberOfStates() + " states");
            
            // Step 2: Create Σ* automaton (0-255 characters)
            System.out.println("\nStep 2: Creating Σ* automaton (0-255 characters)...");
            Automaton sigmaStarAutomaton = createSigmaStarAutomaton();
            System.out.println("Σ* automaton created with " + sigmaStarAutomaton.getNumberOfStates() + " states");
            
            // Step 3: Intersect with Σ* automaton
            System.out.println("\nStep 3: Intersecting regex automaton with Σ*...");
            Automaton intersectedAutomaton = regexAutomaton.intersection(sigmaStarAutomaton);
            System.out.println("Intersected automaton has " + intersectedAutomaton.getNumberOfStates() + " states");
            
            // Step 4: Minimize the automaton
            System.out.println("\nStep 4: Minimizing automaton...");
            intersectedAutomaton.minimize();
            System.out.println("Minimized automaton has " + intersectedAutomaton.getNumberOfStates() + " states");
            
            // // Step 5 (maybe not needed?): Complete the DFA (add garbage state for missing transitions)
            // System.out.println("\nStep 5: Completing DFA (adding garbage state for missing transitions)...");
            // intersectedAutomaton = completeDfa(intersectedAutomaton);
            // System.out.println("Completed DFA has " + intersectedAutomaton.getNumberOfStates() + " states");
            
            // Step 6: Serialize to JSON
            System.out.println("\nStep 6: Serializing to JSON...");
            AutomatonSerializer.serializeToJson(intersectedAutomaton, SAMPLE_REGEX, OUTPUT_FILENAME);
            
            System.out.println("\nDemo completed successfully!");
            
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
    
    /**
     * Completes a DFA by adding a garbage state for missing transitions
     * to ensure all states have transitions for the full alphabet (0-255)
     */
    private static Automaton completeDfa(Automaton automaton) {
        // Clone the automaton to avoid modifying the original
        Automaton completed = automaton.clone();
        
        // Create garbage state (non-accepting)
        State garbageState = new State();
        garbageState.setAccept(false);
        
        // Add self-loop on garbage state for all characters
        garbageState.addTransition(new Transition((char)0, (char)255, garbageState));
        
        boolean garbageStateNeeded = false;
        
        // For each state, check if it has complete transitions for 0-255
        Set<State> allStates = new HashSet<>(completed.getStates());
        for (State state : allStates) {
            List<Transition> transitions = new ArrayList<>(state.getTransitions());
            
            // Sort transitions by character range
            transitions.sort((a, b) -> Character.compare(a.getMin(), b.getMin()));
            
            // Find gaps in character coverage and add transitions to garbage state
            char currentChar = 0;
            for (Transition t : transitions) {
                // If there's a gap before this transition, add transition to garbage state
                if (currentChar < t.getMin()) {
                    state.addTransition(new Transition(currentChar, (char)(t.getMin() - 1), garbageState));
                    garbageStateNeeded = true;
                }
                currentChar = (char)(t.getMax() + 1);
                if (currentChar == 0) break; // Overflow protection
            }
            
            // If there's a gap after the last transition, add transition to garbage state
            if (currentChar <= 255) {
                state.addTransition(new Transition(currentChar, (char)255, garbageState));
                garbageStateNeeded = true;
            }
        }
        
        // Only add garbage state if it's actually needed
        if (garbageStateNeeded) {
            // The garbage state is automatically added when we create transitions to it
            System.out.println("Added garbage state for missing transitions");
        } else {
            System.out.println("DFA was already complete, no garbage state needed");
        }
        
        completed.setDeterministic(true);
        return completed;
    }
} 