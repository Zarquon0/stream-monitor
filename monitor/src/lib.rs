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
                    if input == *byte { return next_state.clone() } 
                },
                TransitionDesc::Range(start_byte, end_byte, next_state) => {
                    if input >= *start_byte || input <= *end_byte { return next_state.clone() }
                },
            }
        }
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