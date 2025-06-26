use super::*;

//Helpers
fn wrap_pattern(pat: &str) -> Option<String> { Option::Some(String::from(pat)) }
fn cmd_vec(raw_cmd: &str) -> Vec<String> { raw_cmd.split_whitespace().map(str::to_string).collect() }
    
//Tests
#[test]
fn simple_match() {
    let command = cmd_vec("echo hello world");
    let pattern = wrap_pattern("hello world");
    assert_eq!(monitor(command, Option::None, pattern), String::from("hello world"));
}
#[test]
#[should_panic]
fn simple_fail() {
    let command = cmd_vec("echo hello world");
    let pattern = wrap_pattern("helo world");
    monitor(command, Option::None, pattern);
}
#[test]
fn basic_regex_match() {
    let command = cmd_vec("echo hello world.");
    let pattern = wrap_pattern(r"[a-z]+ [a-z]*\.");
    assert_eq!(monitor(command, Option::None, pattern), String::from("hello world."));
}
#[test]
#[should_panic]
fn basic_regex_fail() {
    let command = cmd_vec("echo hello w0rld.");
    let pattern = wrap_pattern(r"[a-z]+ [a-z]*\.");
    monitor(command, Option::None, pattern);
}
#[test]
fn complex_regex_match() {
    let command = cmd_vec("ls -l");
    let pattern = wrap_pattern(r"(total [0-9]+)|([drwxr@-]+ +[0-9]+ +[^ ]+ +[^ ]+ +[0-9]+ +[a-zA-Z]+ +[0-9]+ +[0-9:]+ +.+)");
    monitor(command, Option::None, pattern); //Can't assert equivalence b/c variable output - not panicking should be good enough!
}
#[test]
#[should_panic]
fn complex_regex_fail() {
    let command = cmd_vec("ps -f");
    let pattern = wrap_pattern(r"(UID( )+PID( )+PPID( )+C( )+STIME( )+TTY( )+TIME( )+CMD)|(([0-9a-zA-Z_]+|-)( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[0-9]+( )+[a-z0-9/?]+[^ ]+[0-9][0-9:]+( )+.+)");
    monitor(command, Option::None, pattern);
}
#[test]
fn complex_command() {
    let command = cmd_vec("ifconfig | grep 'inet ' | grep -v 127.0.0.1 | cut -f  2");
    let pattern = wrap_pattern(r"^ *(~(inet +)|(inet +([0-9]+\.){3}[0-9]+))");
    monitor(command, Option::None, pattern);
}
#[test]
#[should_panic]
fn invalid_command() {
    let command = cmd_vec("not_a_command");
    let pattern = wrap_pattern(r".*");
    monitor(command, Option::None, pattern);
}
#[test]
#[should_panic]
fn invalid_command_usage() {
    let command = cmd_vec("grep");
    let pattern = wrap_pattern(r".*");
    monitor(command, Option::None, pattern);
}