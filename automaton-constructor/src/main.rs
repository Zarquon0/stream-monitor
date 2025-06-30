use std::{
    collections::{ HashMap, HashSet },
    fs::File, io::BufReader, 
    path::PathBuf, 
};
use clap::Parser;
use serde::Deserialize;
use serde_json;
use monitor::{Dfa, TransitionDesc, TransitionTable};
use regex_automata::util::primitives::StateID;

#[cfg(test)]
mod tests;

//Structs for initial representation of DFA from JSON

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
    let _dfa = dfa_from_json(args.json_file).expect("Error creating DFA - check file path and format of file");
    println!("Success!")
    //let path = dfa.serialize();
    //if !args.suppress { println!("Successfully created and serialized DFA at {:?}!", path) }
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