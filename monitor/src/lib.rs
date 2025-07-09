use std::{
    collections::{HashMap, HashSet}, 
    fs::{create_dir, remove_dir_all, File}, 
    io::{self, BufWriter, BufReader, Read, Write}, 
    path::PathBuf,
    u8::{MAX, MIN}
};
use regex_automata::{
    dfa::{Automaton, StartError}, 
    util::{
        primitives::{PatternID, StateID}, //These two are wrapped u32s
        start::Config,
    }
};
use bitcode::{self, Encode, Decode};
use serde;
use serde_json;

//Expose timer for use by any crate 
pub mod timer;

pub enum TransitionDesc {
    Match(u8, StateID), //If an input byte == the sole u8 -> transition to StateID
    Range(u8, u8, StateID), //If an input byte is >= the first u8 and <= the second u8 -> transition to StateID
}
pub type TransitionTable = HashMap<StateID, Vec<TransitionDesc>>;

pub struct Dfa {
    start_state: StateID,
    match_states: HashSet<StateID>,
    transition_table: TransitionTable,
    dead_state: StateID,
}
impl Dfa {
    pub fn new(start_state: StateID, match_states: HashSet<StateID>, mut transition_table: TransitionTable) -> Self {
        let dead_state = StateID::must(0);
        //Check to make sure the transition table doesn't already define a state at dead state's ID and add dead state behavior
        assert!(transition_table.get(&dead_state).is_none(), "Transition table already had behavior defined for StateID used as dead state");
        transition_table.insert(
            dead_state.clone(), 
            vec![TransitionDesc::Range(MIN, MAX, dead_state.clone())]
        );
        Dfa { start_state, match_states, transition_table, dead_state }
    }
    pub fn serialize(self) -> PathBuf { 
        let serializable_self = SerDfa::from(self);
        serializable_self.serialize().expect("Failed to serialize DFA")
    }
    pub fn deserialize(path: PathBuf) -> Self { 
        let serializable_self = SerDfa::deserialize(&path).expect(format!("Failed to deserialize DFA from {:?}", path).as_str());
        serializable_self.to_dfa()
    }
    pub fn deserialize_from_json(path: PathBuf) -> Self { dfa_from_json(path).expect("Failed deserializing DFA from JSON - check file path") }
    pub fn clean_cache() { SerDfa::clean_ser_dir(); }
}
unsafe impl Automaton for Dfa {
    fn next_state(&self, current: StateID, input: u8) -> StateID {
        for trans_desc in self.transition_table.get(&current).expect("Current state not in transition table - misconstructed DFA") {
            match trans_desc {
                TransitionDesc::Match(byte, next_state) => { 
                    if input == *byte { 
                        //dbg!(format!("Next state: {:?}", next_state));
                        return next_state.clone() 
                    } 
                },
                TransitionDesc::Range(start_byte, end_byte, next_state) => {
                    if input >= *start_byte && input <= *end_byte { 
                        //dbg!(format!("Next state: {:?}", next_state));
                        return next_state.clone() 
                    }
                },
            }
        }
        //dbg!(format!("Next state: {:?}", self.dead_state));
        self.dead_state
    }
    unsafe fn next_state_unchecked(&self, current: StateID, input: u8) -> StateID {
        self.next_state(current, input) //For now...
    }
    fn next_eoi_state(&self, current: StateID) -> StateID { current } //I think... I assume DFAs coming in will not have this special EOI feature
    fn start_state(&self, _config: &Config) -> Result<StateID, StartError> { Ok(self.start_state) } //We don't need fancy configuration stuff - our searches are always anchored!
    fn is_special_state(&self, id: StateID) -> bool { self.is_dead_state(id) || self.is_match_state(id) || self.is_start_state(id) } //We can exclude quit and accel for now
    fn is_dead_state(&self, id: StateID) -> bool { id == self.dead_state }
    fn is_quit_state(&self, _id: StateID) -> bool { false } //I don't think we need quit states for our use case...
    fn is_match_state(&self, id: StateID) -> bool { self.match_states.contains(&id) }
    fn is_start_state(&self, id: StateID) -> bool { id == self.start_state }
    fn is_accel_state(&self, _id: StateID) -> bool { false } //For now...
    fn pattern_len(&self) -> usize { 1 } //The monitor should never search for more than one pattern
    fn match_len(&self, id: StateID) -> usize { 
        if self.is_match_state(id) { 1 } else { 0 } //See above ^^^
    }
    fn match_pattern(&self, id: StateID, _index: usize) -> PatternID { 
        //Not entirely sure if this is totally correct, but the docs say this method won't get called for 
        //single pattern Automatons, so this should be good enough
        if self.is_match_state(id) { PatternID::must(0) } else { panic!("ID {:?} is not a match state", id) }
    }
    fn has_empty(&self) -> bool { self.is_match_state(self.start_state) }
    fn is_utf8(&self) -> bool { false } //Our automaton will only have to address ASCII characters
    fn is_always_start_anchored(&self) -> bool { true } //All patterns are anchored at both ends
}

//
//SERIALIZATION OBJECTS
//

const CACHE_DIR: &str = "serialized-dfa-cache";

#[derive(Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
enum STD {
    Match(u8, u32),
    Range(u8, u8, u32)
}

#[derive(Encode, Decode, Debug)]
struct SerDfa {
    start_state: u32,
    match_states: HashSet<u32>,
    transition_table: HashMap<u32, Vec<STD>>,
    dead_state: u32,
}
impl SerDfa {
    fn from(dfa: Dfa) -> Self {
        let mut new_table = HashMap::new();
        for (key, val) in dfa.transition_table {
            let new_val: Vec<STD> = val
                .iter().map(|td| match *td {
                    TransitionDesc::Match(b, sid) => STD::Match(b, sid.as_u32()),
                    TransitionDesc::Range(b1, b2, sid) => STD::Range(b1, b2, sid.as_u32())
                }).collect();
            new_table.insert(key.as_u32(), new_val);
        }
        Self {
            start_state: dfa.start_state.as_u32(),
            match_states: dfa.match_states.iter().map(|sid| sid.as_u32()).collect(),
            transition_table: new_table,
            dead_state: dfa.dead_state.as_u32(),
        }
    }
    fn to_dfa(self) -> Dfa {
        let mut new_table: TransitionTable = HashMap::new();
        for (key, val) in self.transition_table {
            let new_val: Vec<TransitionDesc> = val
                .iter().map(|td| match *td {
                    STD::Match(b, sid) => TransitionDesc::Match(b, StateID::must(sid as usize)),
                    STD::Range(b1, b2, sid) => TransitionDesc::Range(b1, b2, StateID::must(sid as usize))
                }).collect();
            new_table.insert(StateID::must(key as usize), new_val);
        }
        Dfa {
            start_state: StateID::must(self.start_state as usize), 
            match_states: self.match_states.iter().map(|sid| StateID::must(*sid as usize)).collect(),
            transition_table: new_table,
            dead_state: StateID::must(self.dead_state as usize),
        }
    }
    fn serialize(&self) -> io::Result<PathBuf> {
        //Encode into bytes
        let bytes: Vec<u8> = bitcode::encode(self);
        //Create file with name based on hash of encoded bytes - WARNING: file names are non-deterministic due to non-deterministic iterating order of hashmaps/hashsets
        let hash = blake3::hash(&bytes);
        let cache_dir = proj_root().join(CACHE_DIR);
        if !cache_dir.exists() { create_dir(&cache_dir).expect("Failed to create dfa cache dir"); }
        let path = cache_dir.join(format!("sdfa-{}.bc", hash.to_hex()[..8].to_string()));
        let file = File::create(&path).expect(format!("Couldn't create file at path: {:?}", path).as_str());
        //Write bytes to file
        let mut writer = BufWriter::new(file);
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(path)
    }
    fn deserialize(path: &PathBuf) -> io::Result<Self> {
        let file = File::open(path).expect(format!("Couldn't open file at path: {:?}", path).as_str());
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(bitcode::decode(&buffer).expect("Failed to decode"))
    }
    fn clean_ser_dir() {
        let cache_dir = proj_root().join(CACHE_DIR);
        if cache_dir.exists() { 
            remove_dir_all(cache_dir).expect("Failed to delete serialized DFA cache directory"); 
        }
    }
}

fn proj_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

//
//JSON PARSING OBJECTS
//

#[derive(serde::Deserialize)]
struct JsonDfa {
    start_state: usize, //Unfortunately, there's no way to get serde to deserialize directly to StateIDs because StateIDs have private fields
    match_states: Vec<usize>,
    transition_table: Vec<JsonTransition>,
}
#[derive(serde::Deserialize)]
struct JsonTransition {
    curr_state: usize,
    range_start: u8,
    range_end: u8,
    next_state: usize, 
}

/// Given a path to a JSON file containing properly formatted information about a DFA,
/// attempts to construct a Dfa object out of that information
fn dfa_from_json(json_path: PathBuf) -> std::io::Result<Dfa> { 
    let json_file = File::open(json_path)?;
    let json_dfa: JsonDfa = serde_json::from_reader(BufReader::new(json_file))?;
    Ok(Dfa::new(
        StateID::must(json_dfa.start_state),
        json_dfa.match_states.iter().map(|sid| StateID::must(*sid)).collect::<HashSet<StateID>>(),
        convert_json_transitions(json_dfa.transition_table)
    ))
 }

 fn convert_json_transitions(jtrans: Vec<JsonTransition>) -> TransitionTable {
    //Convert all JsonTransitions to TransitionTable entries
    let mut trans_table = HashMap::new();
    let mut next_states = HashSet::new();
    for trans in jtrans {
        let trans_desc_vec = trans_table
            .entry(StateID::must(trans.curr_state)) 
            .or_insert(Vec::new());
        let next_state = StateID::must(trans.next_state);
        next_states.insert(next_state.clone());
        let new_trans_desc = if trans.range_start == trans.range_end { TransitionDesc::Match(trans.range_start, next_state) } 
            else { TransitionDesc::Range(trans.range_start, trans.range_end, next_state) };
        trans_desc_vec.push(new_trans_desc);
    }
    //Check to make sure that all states do actually have an entry in the table - this is not garaunteed be default!
    let next_states: Vec<StateID> = next_states.into_iter().collect();
    for state in next_states {
        if let None = trans_table.get(&state) { 
            trans_table.insert(state, Vec::new()); //Empty vector because all transitions from these states lead to dead state
        }
    }
    trans_table
 }