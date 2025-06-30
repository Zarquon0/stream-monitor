use std::{
    collections::{HashMap, HashSet}, path::PathBuf, u8::{MAX, MIN}
};
use regex_automata::{
    dfa::{Automaton, StartError}, util::{
        primitives::{PatternID, StateID}, //These two are wrapped u32s
        start::Config,
    }
};
//use rkyv::{Archive, Serialize, Deserialize, with::{AsHashMap, AsHashSet}};

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
    pub fn serialize(&self) -> PathBuf { todo!() }
    pub fn deserialize(path: PathBuf) -> Self { todo!() }
}
unsafe impl Automaton for Dfa {
    fn next_state(&self, current: StateID, input: u8) -> StateID {
        for trans_desc in self.transition_table.get(&current).expect("Current state not in transition table - misconstructed DFA") {
            match trans_desc {
                TransitionDesc::Match(byte, next_state) => { 
                    if input == *byte { 
                        println!("Next state: {:?}", next_state);
                        return next_state.clone() 
                    } 
                },
                TransitionDesc::Range(start_byte, end_byte, next_state) => {
                    if input >= *start_byte && input <= *end_byte { 
                        println!("Next state: {:?}", next_state);
                        return next_state.clone() 
                    }
                },
            }
        }
        println!("Next state: {:?}", self.dead_state);
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
//TEMPORARILY PORTED OVER JSON PARSING FUNCTIONALITY
//

use std::{fs::File, io::BufReader};
use serde::Deserialize;
use serde_json;

impl Dfa {
    pub fn deserialize1(path: PathBuf) -> Self {
        dfa_from_json(path).unwrap()
    }
}

#[derive(Deserialize)]
struct JsonDfa {
    start_state: usize, //Unfortunately, there's no way to get serde to deserialize directly to StateIDs because StateIDs have private fields
    match_states: Vec<usize>,
    transition_table: Vec<JsonTransition>,
}
#[derive(Deserialize)]
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
    let mut trans_table = HashMap::new();
    for trans in jtrans {
        let trans_desc_vec = trans_table
            .entry(StateID::must(trans.curr_state)) 
            .or_insert(Vec::new());
        let next_state = StateID::must(trans.next_state);
        let new_trans_desc = if trans.range_start == trans.range_end { TransitionDesc::Match(trans.range_start, next_state) } 
            else { TransitionDesc::Range(trans.range_start, trans.range_end, next_state) };
        trans_desc_vec.push(new_trans_desc);
    }
    trans_table
 }


// #[derive(Archive, Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
// struct RkyvStateID(u32);
// impl From<StateID> for RkyvStateID {
//     fn from(id: StateID) -> Self { RkyvStateID(id.as_usize() as u32) }
// }
// impl From<RkyvStateID> for StateID {
//     fn from(id: RkyvStateID) -> Self { StateID::must(id.0 as usize) }
// }

// #[derive(Archive, Serialize, Deserialize)]
// enum RkyvTransitionDesc {
//     Match(u8, RkyvStateID), //If an input byte == the sole u8 -> transition to StateID
//     Range(u8, u8, RkyvStateID), //If an input byte is >= the first u8 and <= the second u8 -> transition to StateID
// }
// type RkyvTransitionTable = HashMap<RkyvStateID, Vec<RkyvTransitionDesc>>;

// fn rkyv_tt(tt: TransitionTable) -> RkyvTransitionTable {
//     let mut new_table: RkyvTransitionTable = HashMap::new();
//     for (key, val) in tt {
//         let new_val: Vec<RkyvTransitionDesc> = val
//             .iter().map(|td| match *td {
//                 TransitionDesc::Match(b, sid) => RkyvTransitionDesc::Match(b, sid.into()),
//                 TransitionDesc::Range(b1, b2, sid) => RkyvTransitionDesc::Range(b1, b2, sid.into())
//             }).collect();
//         new_table.insert(key.into(), new_val);
//     }
//     new_table
// }
// fn unrkyv_rtt(rtt: RkyvTransitionTable) -> TransitionTable {
//     let mut new_table: TransitionTable = HashMap::new();
//     for (key, val) in rtt {
//         let new_val: Vec<TransitionDesc> = val
//             .iter().map(|td| match *td {
//                 RkyvTransitionDesc::Match(b, sid) => TransitionDesc::Match(b, sid.into()),
//                 RkyvTransitionDesc::Range(b1, b2, sid) => TransitionDesc::Range(b1, b2, sid.into())
//             }).collect();
//         new_table.insert(key.into(), new_val);
//     }
//     new_table
// }

// #[derive(Archive, Serialize, Deserialize)]
// //#[archive(check_bytes)]
// struct RkyvDfa {
//     start_state: RkyvStateID,
//     match_states: HashSet<RkyvStateID>,
//     transition_table: HashMap<RkyvStateID, Vec<RkyvTransitionDesc>>,
//     dead_state: RkyvStateID,
// }
// impl RkyvDfa {
//     fn make_dfa(self) -> Dfa {
//         Dfa { 
//             start_state: self.start_state.into(),
//             match_states: self.match_states.iter().map(|sid| *sid.into()).collect(),
//             transition_table: unrkyv_rtt(self.transition_table),
//             dead_state: self.dead_state.into(),
//         }
//     }
// }

// #[derive(Archive, Serialize, Deserialize)]
// struct RDfa {
//     start_state: u32,
//     match_states: HashSet<u32>,
//     transition_table: HashMap<u32, Vec<Thing>>,
//     dead_state: u32,
// }

// #[derive(Archive, Serialize, Deserialize)]
// enum Thing {
//     Match(u8, u32), //If an input byte == the sole u8 -> transition to StateID
//     Range(u8, u8, u32), //If an input byte is >= the first
// }