use std::{collections::HashMap, path::PathBuf};
use clap::Parser;
use regex_automata::{
    dfa::{Automaton, StartError},
    util::{
        primitives::{StateID, PatternID}, //These two are wrapped u32s
        start::Config,
    },
};

#[cfg(test)]
mod tests;

#[derive(Eq, Hash, PartialEq)]
struct TransPair(StateID, u8); //A current state (StateID) and an input (byte - u8) define a next state (StateID)

struct Dfa {
    start_states: Vec<StateID>,
    match_states: Vec<StateID>,
    transition_table: HashMap<TransPair, StateID>,
}
unsafe impl Automaton for Dfa {
    fn next_state(&self, current: StateID, input: u8) -> StateID {
        match self.transition_table.get(&TransPair(current, input)) {
            Some(sid) => sid.clone(),
            None => panic!("State-input pair not accounted for in transition table."),
        }
    }
    unsafe fn next_state_unchecked(&self, current: StateID, input: u8) -> StateID { todo!() }
    fn next_eoi_state(&self, current: StateID) -> StateID { todo!() }
    fn start_state(&self, config: &Config) -> Result<StateID, StartError> { todo!() }
    fn is_special_state(&self, id: StateID) -> bool { todo!() }
    fn is_dead_state(&self, id: StateID) -> bool { todo!() }
    fn is_quit_state(&self, id: StateID) -> bool { todo!() }
    fn is_match_state(&self, id: StateID) -> bool { todo!() }
    fn is_start_state(&self, id: StateID) -> bool { todo!() }
    fn is_accel_state(&self, id: StateID) -> bool { todo!() }
    fn pattern_len(&self) -> usize { 1 } //The monitor should never search for more than one pattern
    fn match_len(&self, id: StateID) -> usize { 
        if self.is_match_state(id) { 1 } else { 0 } //See above ^^^
    }
    fn match_pattern(&self, id: StateID, _index: usize) -> PatternID { 
        //Not entirely sure if this is totally correct, but the docs say this method won't get called for 
        //single pattern Automatons, so this should be good enough
        if self.is_match_state(id) { PatternID::new(0).unwrap() } else { panic!("ID {:?} is not a match state", id) }
    }
    fn has_empty(&self) -> bool { todo!() }
    fn is_utf8(&self) -> bool { todo!() }
    fn is_always_start_anchored(&self) -> bool { true } //All patterns are anchored at both ends
}
 
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Suppress's all output to stdout 
    #[arg(short, long, default_value_t=false)]
    suppress: bool,
    ///File path to file containing input to check - if not specified, monitor will instead look to stdin
    #[arg(required(true))]
    json_file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let dfa = dfa_from_json(args.json_file).expect("Error creating DFA - check file path and format of file");
    let path = serialize_dfa(dfa);
    if !args.suppress { println!("Successfully created and serialized DFA at {:?}!", path) }
}

/// Given a path to a JSON file containing properly formatted information about a DFA,
/// attempts to construct a Dfa object out of that information
fn dfa_from_json(json_path: PathBuf) -> std::io::Result<Dfa> { todo!() }

/// Serializes the input dfa and stores it in a file, returning the new file's path
fn serialize_dfa(dfa: Dfa) -> PathBuf { todo!() }