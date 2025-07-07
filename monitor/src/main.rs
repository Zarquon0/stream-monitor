use clap::Parser;
use regex_automata::{ dfa::{dense::DFA, Automaton}, HalfMatch, Input };
use atty::{self, Stream};
use std::path::PathBuf;
use std::io::{self, BufRead, BufReader};
use std::process::exit;
use std::fs::{self, File};
use std::env;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use monitor::Dfa;

#[cfg(test)]
mod tests;



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
    ///On a failed validation, instead of panicking, will send a SIGTERM signal to the PID stored at env variable
    ///MONITOR_TARGET_PID
    #[arg(short, long, default_value_t=false)]
    trap: bool,
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
    //Validate the stream and handle validation failure behavior
    if let Err(e) = validate_stream(input_stream, dfa) {
        let msg = match e {
            ValidationFailure::Partial(line) => format!("Validation failed (partial match)\nIncriminating line: {}", line),
            ValidationFailure::Whole(line) => format!("Validation failed\nIncriminating line: {}", line),
        };
        if args.trap { kill_shell(msg.as_str()).expect("Trap not properly set up (and validation failed)") }
        else { panic!("{}", msg) }
    }
    
}

enum ValidationFailure {
    Partial(String),
    Whole(String),
}

/// Given a stream and a DFA, walks the DFA over the stream, writing each line of the stream to stdout as it
/// validates
fn validate_stream(stream: Box<dyn BufRead>, dfa: Box<dyn Automaton>) -> Result<(), ValidationFailure> {
    for line in stream.lines() {
        let line = line.expect("Error grabbing next line");
        //let _state = dfa.start_state(&Config::new()).expect("Couldn't bring DFA to start state");
        match dfa.try_search_fwd(&Input::new(&line.as_bytes())).expect("DFA search errored") {
            Some(mtch) => {
                if mtch == HalfMatch::must(0, line.len()) { println!("{}", line.as_str()) } //Write line to stdout - done line by line to preserve streaming
                else { return Err(ValidationFailure::Partial(line)) }
            },
            None => return Err(ValidationFailure::Whole(line))
        }
    }
    Ok(())
}

/// Assuming the appropriate environment variables and trap are set, sends a message to be print 
/// and a kill signal to the parent shell process
fn kill_shell(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    //Read environment variables
    let file_path = env::var("MONITOR_MESSAGE_FILE").map_err(|_| "MONITOR_MESSAGE_FILE not set")?;
    let pid_str = env::var("MONITOR_TARGET_PID").map_err(|_| "MONITOR_TARGET_PID not set")?;
    let pid = pid_str.parse().map_err(|_| "MONITOR_TARGET_PID not properly set (couldn't parse to an i32)")?;
    //Write message to temp file and send the kill signal
    fs::write(&file_path, message)?;
    signal::kill(Pid::from_raw(pid), Signal::SIGUSR1)?;
    Ok(())
}