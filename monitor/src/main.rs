//use regex::Regex;
use clap::Parser;
use regex_automata::{ dfa::{dense::DFA, Automaton}, HalfMatch, Input };// MatchKind, util::start::Config };
use atty::{self, Stream};
use std::path::PathBuf;
use std::io::{self, BufRead, BufReader};
use std::process::exit;
use std::fs::File;

use monitor::Dfa;

#[cfg(test)]
mod tests;

// mod timer;
// use crate::timer::Timer;


//OUTSTANDING TASKS/QUESTIONS
//Test performance and speed up!
//  -  Make performance test file - DONE (more or less)
//  -  Replace regex library with quicker ripgrep regex engine/custom engine? Update: regex-automata is solution!
//Ensure compliance with the entire range of stream types out there (account for any divergence with typical regex) - DONE: there is no divergence
//  -  Are speedups acheivable if regex matching takes into account any limitations in the expressivity of the stream
//     types w.r.t. standard regex? 
//  -  Are stream types always fully specific (ie. could always be written as '^<stream_type>$')? - YES
//Create Makefile to streamline binary creation/testing - DONE
//Create README with explanation and usage instructions
//Create more/better tests as additions continue
//Figure out how input validation should work
//Create external process execution monitor to halt entire script's execution upon monitor failure? 
//Figure out how to handle errors that crop up from running input commands themselves
//Monitor output from each individual member of a long piped command for more precise debugging?
//  -  Related: Ensure system works with pipes (currently, it doesn't)... And redirects... If we don't want spin up a whole shell here, we're gonna need to get creative
//Pipe validated output to stdout as received to maintain the benefits of a stream-based approach (instead of println!)

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///File path to serialized DFA - if not specified, no_validation must be true
    #[arg(short, required(false))]
    dfa_path: Option<PathBuf>,
    ///No validation will be performed (DFA defaults to a .* matcher) - mainly exists for development purposes
    #[arg(long, default_value_t=false)]
    no_validation: bool,
    ///File path to file containing input to check - if not specified, monitor will instead look to stdin
    #[arg(required(false))]
    input_file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    //Parse input stream object and DFA from arguments
    let input_stream: Box<dyn BufRead> = match args.input_file {
        Some(path) => { //Input file provided
            let input_file = File::open(path).expect("DFA path invalid");
            Box::new(BufReader::new(input_file))
        },
        None => { //No input file provided, default to stdin
            if atty::is(Stream::Stdin) { //No input stream provided - is attached to interactive terminal
                panic!("No input stream piped in or provided via file") 
            } else { //Input stream being piped in - return that stream
                Box::new(io::stdin().lock())
            }
        }
    };
    let dfa: Box<dyn Automaton> = match args.dfa_path {
        Some(path) => { //Currenty only supported for DFA's built through regex_automata!
            // let dfa_bytes = fs::read(path).expect("Path to DFA invalid.");
            // let dfa = DFA::from_bytes(&dfa_bytes)
            //     .expect("Unable to deserialize DFA. Ensure provided file is a serialized DFA build from regex_automata.")
            //     .0.to_owned(); //Gets around borrow of dfa_bytes
            let dfa = Dfa::deserialize(path);
            Box::new(dfa)
        },
        None => {
            if args.no_validation { Box::new(DFA::new(r".*").unwrap()) }
            else { 
                eprintln!("No DFA specified. Must either specify a DFA (via -d) or set --no-validation.");
                exit(1)
            }
        }
    };
    //Validate the stream and print the output back to stdout (assuming the stream is valid)
    let validated_output = validate_stream(input_stream, dfa);
    println!("{}", validated_output);
    
}

/// Given a stream and a DFA, walks the DFA over the stream, returning the entirety of the stream if it fully matches
/// the DFA and panicking if it does not.
fn validate_stream(stream: Box<dyn BufRead>, dfa: Box<dyn Automaton>) -> String {
    let mut output = String::new();
    for line in stream.lines() {
        let line = line.expect("Error grabbing next line");
        //let _state = dfa.start_state(&Config::new()).expect("Couldn't bring DFA to start state");
        match dfa.try_search_fwd(&Input::new(&line.as_bytes())).expect("DFA search errored") {
            Some(mtch) => {
                if mtch == HalfMatch::must(0, line.len()) { output += line.as_str() }
                else { panic!("Validation failed (partial match).\nIncriminating line: {}", line)}
            },
            None => panic!("Validation failed.\nIncriminating line: {}", line)
        }
    }
    output
}

// /// Given a shell command (represented as a vector of Strings), ensures that the input to that command matches any
// /// specified regex ipattern (coming soon!) and that the output to that command matches any specified regex opattern,
// /// returning the command's output if both validations are satisfied and panicking if not.
// fn monitor(command: Vec<String>, _ipattern: Option<String>, opattern: Option<String>) -> String {
//     let _monitor_timer = Timer::new("Full Monitor");
//     //Run command and retrieve output stream
//     let _command_timer = Timer::new("Run Command");
//     let (mut buffed_stream, child_pid) = run_command(command);
//     drop(_command_timer);
//     //Validate output
//     let _validation_timer = Timer::new("Output Validation");
//     let output = if let Option::Some(opattern) = opattern { validate_stream(buffed_stream, opattern) } 
//     else { 
//         let mut output_buf = String::new();
//         buffed_stream.read_to_string(&mut output_buf).expect("Reading output buffer to a String failed");
//         output_buf
//     };
//     drop(_validation_timer);
//     waitpid(child_pid, None).unwrap();
//     output
// }

// /// Verifies that each line of the input buffered stream matches the input regex pattern, returning the full
// /// output if so and panicking if not.
// fn validate_stream(buffed_stream: BufReader<File>, pattern: String) -> String {
//     let pattern = Regex::new(&pattern).expect("Provided regular expression invalid");
//     //Line by line matching approach
//     let mut full_output = String::new();
//     for line_result in buffed_stream.lines() {
//         let line = line_result.unwrap();
//         if !pattern.is_match(&line) { panic!("Command output did not match pattern.\nPattern: {:?}\n Incriminating line of output: {:?}", pattern, line) }
//         full_output += line.as_str();
//     }
//     full_output
// }