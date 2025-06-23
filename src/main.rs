use regex;
use clap::Parser;
use std::process::Command;
// use pprof::ProfilerGuard;
// use std::fs::File;

mod timer;
use crate::timer::Timer;

//OUTSTANDING TASKS/QUESTIONS
//Test performance and speed up!
//  -  Make performance test file
//  -  Replace regex library with quicker ripgrep regex engine/custom engine?
//Ensure compliance with the entire range of stream types out there (account for any divergence with typical regex)
//  -  Are speedups acheivable if regex matching takes into account any limitations in the expressivity of the stream
//     types w.r.t. standard regex?
//  -  Are stream types always fully specific (ie. could always be written as '^<stream_type>$')?
//Create Makefile to streamline binary creation/testing - DONE
//Create README with explanation and usage instructions
//Create more/better tests as additions continue
//Add command line options?
//Create external process execution monitor to halt entire script's execution upon monitor failure? 
//Figure out how to handle errors that crop up from running input commands themselves
//Monitor output from each individual member of a long piped command for more precise debugging?

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    reg_exp: String,

    #[arg()]
    command_parts: Vec<String>
}

fn main() {
    //let guard = ProfilerGuard::new(100).unwrap();
    let _total_time = Timer::new("Total");

    let args = Args::parse();
    let command = args.command_parts.join(" ");
    let output = monitor(command, args.reg_exp);
    println!("{:?}", output);

    // if let Ok(report) = guard.report().build() {
    //     let mut file = File::create("flamegraph.svg").unwrap();
    //     report.flamegraph(&mut file).unwrap();
    //     println!("Flamegraph written to flamegraph.svg");
    // }
}

fn monitor(command: String, pattern: String) -> String {
    //Run command and retrieve output to stdout
    let command_timer = Timer::new("Shell Command");
    let command_out = Command::new("sh")
        .arg("-c").arg(&command)
        .output()
        .expect("Command no run :(");
    let stdout = String::from_utf8_lossy(&command_out.stdout);
    drop(command_timer);
    //Act according to whether output matches expected pattern
    let _regex_timer = Timer::new("Regex Match");
    let output_match = regex::Regex::new(&pattern).expect("Provided regular expression invalid");
    if output_match.is_match(&stdout) { stdout.to_string() }
    else { panic!("Command output did not match pattern.\nPattern: {:?}\nCommand output: {:?}", pattern, stdout) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn simple_match() {
        let command = String::from("echo hello world");
        let pattern = String::from("hello world");
        assert_eq!(monitor(command, pattern), String::from("hello world\n"));
    }
    #[test]
    #[should_panic]
    fn simple_fail() {
        let command = String::from("echo hello world");
        let pattern = String::from("helo world");
        monitor(command, pattern);
    }
    #[test]
    fn basic_regex_match() {
        let command = String::from("echo hello world.");
        let pattern = String::from(r"[a-z]+ [a-z]*\.");
        assert_eq!(monitor(command, pattern), String::from("hello world.\n"));
    }
    #[test]
    #[should_panic]
    fn basic_regex_fail() {
        let command = String::from("echo hello w0rld.");
        let pattern = String::from(r"[a-z]+ [a-z]*\.");
        monitor(command, pattern);
    }
    #[test]
    fn complex_regex_match() {
        let command = String::from("ls -l");
        let pattern = String::from(r"(total [0-9]+)|([drwxr-]+ +[0-9]+ +[^ ]+ +[^ ]+ +[0-9]+ +[^ ]+ +[0-9]+ +[a-zA-Z]+ +[0-9]+ + [0-9:]+ +.+)");
        monitor(command, pattern); //Can't assert equivalence b/c variable output - not panicking should be good enough!
    }
    #[test]
    #[should_panic]
    fn complex_regex_fail() {
        let command = String::from("ps -f");
        let pattern = String::from(r"(UID( )+PID( )+PPID( )+C( )+STIME( )+TTY( )+TIME( )+CMD)|(([0-9a-zA-Z_]+|-)( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[a-z0-9/?]+[^ ]+[0-9][0-9:]+( )+.+)");
        monitor(command, pattern);
    }
    #[test]
    fn complex_command() {
        let command = String::from("ifconfig | grep 'inet ' | grep -v 127.0.0.1 | cut -f  2");
        let pattern = String::from(r"^ *(~(inet +)|(inet +([0-9]+\.){3}[0-9]+))");
        monitor(command, pattern);
    }
    #[test]
    #[should_panic]
    fn invalid_command() {
        let command = String::from("not_a_command");
        let pattern = String::from(r".*");
        monitor(command, pattern);
    }
    #[test]
    #[should_panic]
    fn invalid_command_usage() {
        let command = String::from("grep");
        let pattern = String::from(r".*");
        monitor(command, pattern);
    }

}
