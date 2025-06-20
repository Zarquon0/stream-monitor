use regex;
use clap::Parser;
use std::process::Command;

//OUTSTANDING TASKS/QUESTIONS
//Test performance and speed up!
//  -  Make performance test file
//  -  Replace regex library with quicker ripgrep regex engine/custom engine?
//Ensure compliance with the entire range of stream types out there (account for any divergence with typical regex)
//  -  Are speedups acheivable if regex matching takes into account any limitations in the expressivity of the stream
//     types w.r.t. standard regex?
//  -  Are stream types always fully specific (ie. could always be written as '^<stream_type>$')?
//Create Makefile to streamline binary creation/testing 
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
    let args = Args::parse();
    let command = args.command_parts.join(" ");
    let output = monitor(command, args.reg_exp);
    println!("{:?}", output);
}

fn monitor(command: String, pattern: String) -> String {
    //Run command and retrieve output to stdout
    let command_out = Command::new("sh")
        .arg("-c").arg(&command)
        .output()
        .expect("Command no run :(");
    let stdout = String::from_utf8_lossy(&command_out.stdout);
    //Act according to whether output matches expected pattern
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
    fn complex_command() {
        let command = String::from("ifconfig | grep 'inet ' | grep -v 127.0.0.1 | cut -f  2");
        let pattern = String::from(r"^ *(~(inet +)|(inet +([0-9]+\.){3}[0-9]+))");
        monitor(command, pattern); //Can't assert equivalence b/c variable output - not panicking should be good enough!
    }

}
