use clap::Parser;
use std::{path::PathBuf, process::{Command, Stdio, exit}};
use std::io::{self, BufReader, Read, Write, BufRead};
use std::thread;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///File path to serialized DFA for streamonitor
    #[arg(short, required(false))]
    dfa_path: Option<PathBuf>,
    ///Regular expression for streamonitor (gets turned into a DFA internally)
    #[arg(short, required(false))]
    regex: Option<String>,
    ///Regular expression for grep
    #[arg(short, required(false))]
    grep: Option<String>,
    ///Regular expression for ripgrep
    #[arg(short = 'R', required(false))]
    ripgrep: Option<String>,
}

fn main() {
    let args = Args::parse();
    //Check to make sure only one kind of operating mode is specified
    let num_args_specified = vec![&args.regex, &args.grep, &args.ripgrep].iter().fold(
        0, 
        |accum, el| if let Some(_) = el { accum + 1 } else { accum }
    ) + (if let Some(_) = args.dfa_path { 1 } else { 0 });
    assert_eq!(num_args_specified, 1, "Must have one (and only one) operating mode specified");
    //Execute proper functionality
    let streamonitor_args = if let Some(path) = args.dfa_path { 
        Some(("./streamonitor".to_string(), vec!["-d".to_string(), path.to_string_lossy().to_string()]))
        //Some(path.clone().to_str().unwrap())
    } else if let Some(regex) = args.regex {
        Some(("./streamonitor".to_string(), vec!["-r".to_string(), regex]))
        //Some(regex.clone().as_str())
    } else if let Some(regex) = args.grep {
        grep_monitor("grep", "-Exc", regex);
        None
    } else if let Some(regex) = args.ripgrep {
        grep_monitor("rg", "-xc", regex);
        None
    } else { panic!("Code is buggy - this branch should be impossible to reach") };
    if let Some((cmd, args)) = streamonitor_args {
        let stat = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit()) // Direct child's stdout to ours
            .stderr(Stdio::inherit()) // Same for stderr
            .status().expect("Command failed to execute");
        exit(stat.code().unwrap_or(1));
    }
}

fn grep_monitor(grep_cmd: &str, options: &str, regex: String) {
    //Run grep command
    let mut grep_cmd = Command::new(grep_cmd)
        .args([options, regex.as_str()]) // -E = use regex matching, -x = match on whole lines, -c = count matching lines
        .stdin(Stdio::piped())
        .stdout(Stdio::piped()) // Direct child's stdout to ours
        .stderr(Stdio::inherit()) // Same for stderr
        .spawn().expect("Command failed to execute");
    //Pass stdin into the grep command while counting its lines
    let mut child_stdin = grep_cmd.stdin.take().expect("Failed to open child stdin");
    let ipt_feeder = thread::spawn(move || -> Vec<String> {
        let stdin_reader = io::stdin().lock();
        //let mut stdout_writer = io::stdout().lock();
        //let mut line_count = 0;
        let mut out_lines = Vec::new();
        for line_res in stdin_reader.lines() {
            let line = line_res.expect("Error reading line of stdin");
            //line_count += 1;
            writeln!(child_stdin, "{}", line).expect("Failed to write to child stdin");
            //writeln!(stdout_writer, "{}", line).expect("Failed to write line to stdout");
            out_lines.push(line);
        }
        out_lines
    });
    //Parse grep's stdout into an integer representing the number of matching lines
    let stdout = grep_cmd.stdout.take().expect("Failed to capture stdout");
    let mut  reader = BufReader::new(stdout);
    let mut grep_res = String::new();
    reader.read_to_string(&mut grep_res).expect("Unable to read grep output to string");
    let num_matching_lines = match grep_res.trim() {
        "" => 0, //For whatever reason, ripgrep returns nothing if there are no matching lines - not sure why
        num_str => num_str.parse::<usize>().expect("Output not an integer - make sure grep is using the -c flag.")
    };
    //Wait till grep process finishes and compare number of matching lines to total number of lines
    let out_lines = ipt_feeder.join().expect("Input feeding thread errored.");
    let num_total_lines = out_lines.len();
    if num_matching_lines != num_total_lines { panic!("Validation Failed") }
    //Print all lines to stdout
    let mut stdout_writer = io::stdout().lock();
    for line in out_lines {
        writeln!(stdout_writer, "{}", line).expect("Failed to write line to stdout");
    }
}