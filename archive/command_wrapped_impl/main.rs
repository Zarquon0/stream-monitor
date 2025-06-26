use regex::Regex;
use clap::Parser;
//Low level shell command utilities
use nix::unistd::{pipe, close, ForkResult, fork, execvp, Pid};
use nix::sys::wait::waitpid;
use nix::libc;
//Standard library helpers
use std::ffi::CString;
use std::io::{ BufRead, BufReader, Read };
use std::os::fd::AsRawFd;
use std::fs::File;
use std::process::exit;

mod timer;
use crate::timer::Timer;

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
    #[arg(short, required(false))]
    input_regex: Option<String>,

    #[arg(short, required(false))]
    output_regex: Option<String>,

    #[arg()]
    command_parts: Vec<String>
}

fn main() {
    let _total_time = Timer::new("Total");
    let args = Args::parse();
    //Alert user of potential unintented functionality if no input/output types are specified
    if let (Option::None, Option::None) = (&args.input_regex, &args.output_regex) { println!("Warning: no type specified for input or output of command, so no validation will take place") }
    let validated_output = monitor(args.command_parts, args.input_regex, args.output_regex);
    println!("{}", validated_output);
}

/// Given a shell command (represented as a vector of Strings), ensures that the input to that command matches any
/// specified regex ipattern (coming soon!) and that the output to that command matches any specified regex opattern,
/// returning the command's output if both validations are satisfied and panicking if not.
fn monitor(command: Vec<String>, _ipattern: Option<String>, opattern: Option<String>) -> String {
    let _monitor_timer = Timer::new("Full Monitor");
    //Run command and retrieve output stream
    let _command_timer = Timer::new("Run Command");
    let (mut buffed_stream, child_pid) = run_command(command);
    drop(_command_timer);
    //Validate output
    let _validation_timer = Timer::new("Output Validation");
    let output = if let Option::Some(opattern) = opattern { validate_stream(buffed_stream, opattern) } 
    else { 
        let mut output_buf = String::new();
        buffed_stream.read_to_string(&mut output_buf).expect("Reading output buffer to a String failed");
        output_buf
    };
    drop(_validation_timer);
    waitpid(child_pid, None).unwrap();
    output
}

/// Runs the input command via fork + exec. Returns a buffered stream reader to the command's output and the PID
/// of the spawn child process for the command.
fn run_command(command: Vec<String>) -> (BufReader<File>, Pid) {
    let (read_fd, write_fd) = pipe().expect("pipe failed"); //~10 micro for pipe
    match unsafe { fork() } { //~500 micro for fork
        Ok(ForkResult::Child) => {
            //Redirect STDOUT to the write end of the pipe and close unnecessary fds
            close(read_fd).unwrap();
            unsafe { //Necessary here because nix::dup2 would require libc::STDOUT_FILENO to be an OwnedFd for extra safety, which we cannot do here without taking unnecessary steps
                let dup_status = libc::dup2(write_fd.as_raw_fd(), libc::STDOUT_FILENO);
                if dup_status == -1 { panic!("Redirecting STDOUT to the write end of the pipe failed: {}", std::io::Error::last_os_error()); }
            }
            close(write_fd).unwrap();
            //Parse and execute command
            //Note: Keep an eye on the .clones - perhaps there's a more efficient way to deal with them (dereferencing is not an option)?
            let cmd = CString::new(command[0].clone()).unwrap();
            let c_args: Vec<CString> = command.iter().map(|s| CString::new(s.clone()).unwrap()).collect();
            execvp(&cmd, &c_args).expect("Command execution failed");
            exit(1) //This line should never run, but Rust's type checker doesn't quite understand how execvp works 
        }
        Ok(ForkResult::Parent { child }) => {
            close(write_fd).unwrap(); //The read_fd will be closed automatically when its owning object is dropped
            // Return a BufReader on the read end of the pipe and the child process's PID
            let pipe_file = std::fs::File::from(read_fd);
            (BufReader::new(pipe_file), child)
        }
        Err(e) => panic!("Fork failed: {}", e)
    }
}

/// Verifies that each line of the input buffered stream matches the input regex pattern, returning the full
/// output if so and panicking if not.
fn validate_stream(buffed_stream: BufReader<File>, pattern: String) -> String {
    let pattern = Regex::new(&pattern).expect("Provided regular expression invalid");
    //Line by line matching approach
    let mut full_output = String::new();
    for line_result in buffed_stream.lines() {
        let line = line_result.unwrap();
        if !pattern.is_match(&line) { panic!("Command output did not match pattern.\nPattern: {:?}\n Incriminating line of output: {:?}", pattern, line) }
        full_output += line.as_str();
    }
    full_output
}