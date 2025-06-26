use super::*;
use regex_automata::dfa::{dense::DFA, Automaton};
use std::io::{BufReader, BufRead};
use std::process::{Command, Stdio};

//Helpers
fn output_stream(raw_cmd: &str) -> Box<dyn BufRead> { 
    let mut command = Command::new("sh")
        .arg("-c").arg(raw_cmd)
        .stdout(Stdio::piped())
        .spawn().expect("Command failed to execute");
    Box::new(BufReader::new(command.stdout.take().unwrap())) 
}
fn dfa_from_pat(pat: &str) -> Box<dyn Automaton> { Box::new(DFA::new(pat).unwrap()) }
    
//TODO: Write tests for main

//validate_stream tests
#[test]
fn simple_match() {
    let stream = output_stream("echo hello world");
    let dfa = dfa_from_pat("hello world");
    assert_eq!(validate_stream(stream, dfa), String::from("hello world"));
}
#[test]
#[should_panic]
fn simple_fail() {
    let stream = output_stream("echo hello world");
    let dfa = dfa_from_pat("helo world");
    validate_stream(stream, dfa);
}
#[test]
fn basic_regex_match() {
    let stream = output_stream("echo hello world.");
    let dfa = dfa_from_pat(r"[a-z]+ [a-z]*\.");
    assert_eq!(validate_stream(stream, dfa), String::from("hello world."));
}
#[test]
#[should_panic]
fn basic_regex_fail() {
    let stream = output_stream("echo hello w0rld.");
    let dfa = dfa_from_pat(r"[a-z]+ [a-z]*\.");
    validate_stream(stream, dfa);
}
#[test]
fn complex_regex_match() {
    let stream = output_stream("ls -l");
    let dfa = dfa_from_pat(r"(total [0-9]+)|([drwxr@-]+ +[0-9]+ +[^ ]+ +[^ ]+ +[0-9]+ +[a-zA-Z]+ +[0-9]+ +[0-9:]+ +.+)");
    validate_stream(stream, dfa); //Can't assert equivalence b/c variable output - not panicking should be good enough!
}
#[test]
#[should_panic]
fn complex_regex_fail() {
    let stream = output_stream("ps -f");
    let dfa = dfa_from_pat(r"(UID( )+PID( )+PPID( )+C( )+STIME( )+TTY( )+TIME( )+CMD)|(([0-9a-zA-Z_]+|-)( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[a-z0-9/?]+[^ ]+[0-9][0-9:]+( )+.+)");
    validate_stream(stream, dfa);
}
#[test]
fn complex_stream() {
    let stream = output_stream("ifconfig | grep 'inet ' | grep -v 127.0.0.1 | cut -f  2");
    let dfa = dfa_from_pat(r"^ *(~(inet +)|(inet +([0-9]+\.){3}[0-9]+))");
    validate_stream(stream, dfa);
}
#[test]
#[should_panic]
fn invalid_stream() {
    let stream = output_stream("not_a_stream");
    let dfa = dfa_from_pat(r".*");
    validate_stream(stream, dfa);
}
#[test]
#[should_panic]
fn invalid_stream_usage() {
    let stream = output_stream("grep");
    let dfa = dfa_from_pat(r".*");
    validate_stream(stream, dfa);
}