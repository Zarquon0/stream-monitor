package com.example.regexdfa;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import dk.brics.automaton.Automaton;
import dk.brics.automaton.State;
import dk.brics.automaton.Transition;

import java.io.File;
import java.io.IOException;
import java.util.*;

/**
 * Serializes a DFA automaton to JSON format
 */
public class AutomatonSerializer {
    
    public static class TransitionEntry {
        @JsonProperty("curr_state")
        private int currState;
        
        @JsonProperty("range_start")
        private int rangeStart;
        
        @JsonProperty("range_end")
        private int rangeEnd;
        
        @JsonProperty("next_state")
        private int nextState;
        
        public TransitionEntry() {}
        
        public TransitionEntry(int currState, int rangeStart, int rangeEnd, int nextState) {
            this.currState = currState;
            this.rangeStart = rangeStart;
            this.rangeEnd = rangeEnd;
            this.nextState = nextState;
        }

        // Getters and setters
        public int getCurrState() { return currState; }
        public void setCurrState(int currState) { this.currState = currState; }
        
        public int getRangeStart() { return rangeStart; }
        public void setRangeStart(int rangeStart) { this.rangeStart = rangeStart; }
        
        public int getRangeEnd() { return rangeEnd; }
        public void setRangeEnd(int rangeEnd) { this.rangeEnd = rangeEnd; }
        
        public int getNextState() { return nextState; }
        public void setNextState(int nextState) { this.nextState = nextState; }
    }
    
    public static class AutomatonJson {
        @JsonProperty("_comment")
        private String comment;
        
        @JsonProperty("start_state")
        private int startState;
        
        @JsonProperty("match_states")
        private List<Integer> matchStates;
        
        @JsonProperty("transition_table")
        private List<TransitionEntry> transitionTable;
        
        public AutomatonJson() {}
        
        public AutomatonJson(String comment, int startState, List<Integer> matchStates, 
                           List<TransitionEntry> transitionTable) {
            this.comment = comment;
            this.startState = startState;
            this.matchStates = matchStates;
            this.transitionTable = transitionTable;
        }

        // Getters and setters
        public String getComment() { return comment; }
        public void setComment(String comment) { this.comment = comment; }
        
        public int getStartState() { return startState; }
        public void setStartState(int startState) { this.startState = startState; }
        
        public List<Integer> getMatchStates() { return matchStates; }
        public void setMatchStates(List<Integer> matchStates) { this.matchStates = matchStates; }
        
        public List<TransitionEntry> getTransitionTable() { return transitionTable; }
        public void setTransitionTable(List<TransitionEntry> transitionTable) { this.transitionTable = transitionTable; }
    }
    
    /**
     * Serializes an automaton to JSON and saves to file
     */
    public static void serializeToJson(Automaton automaton, String regex, String filename) throws IOException {
        // Create state mapping
        Map<State, Integer> stateMap = new HashMap<>();
        List<State> states = new ArrayList<>(automaton.getStates());
        for (int i = 0; i < states.size(); i++) {
            stateMap.put(states.get(i), i + 1); // Start state numbering from 1
        }
        
        // Get start state
        int startState = stateMap.get(automaton.getInitialState());
        
        // Get accepting states
        List<Integer> matchStates = new ArrayList<>();
        for (State state : automaton.getAcceptStates()) {
            matchStates.add(stateMap.get(state));
        }
        Collections.sort(matchStates);
        
        // Build transition table
        List<TransitionEntry> transitionTable = new ArrayList<>();
        for (State state : states) {
            int currStateId = stateMap.get(state);
            for (Transition t : state.getTransitions()) {
                int nextStateId = stateMap.get(t.getDest());
                transitionTable.add(new TransitionEntry(
                    currStateId, (int)t.getMin(), (int)t.getMax(), nextStateId));
            }
        }
        
        // Sort transitions by current state, then by range start
        transitionTable.sort((a, b) -> {
            if (a.currState != b.currState) {
                return Integer.compare(a.currState, b.currState);
            }
            return Integer.compare(a.rangeStart, b.rangeStart);
        });
        
        // Create JSON object
        String comment = "This corresponds to the regular expression '" + regex + "'";
        AutomatonJson automatonJson = new AutomatonJson(comment, startState, matchStates, transitionTable);
        
        // Serialize to JSON
        ObjectMapper mapper = new ObjectMapper();
        mapper.enable(SerializationFeature.INDENT_OUTPUT);
        mapper.writeValue(new File(filename), automatonJson);
        
        System.out.println("Automaton serialized to " + filename);
    }
} 