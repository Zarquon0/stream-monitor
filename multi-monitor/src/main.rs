use clap::Parser;
use os_pipe::pipe;
use std::{path::PathBuf, process::{exit, Command, Stdio}};
use std::io::{self, BufReader, Read, Write, BufRead};
use std::thread;

const MON_BINARY: &str = "streamonitor";

fn proj_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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
    let mon_binary = proj_root().join(MON_BINARY);
    let mon_binary_str = mon_binary.to_str().unwrap();
    let streamonitor_args = if let Some(path) = args.dfa_path { 
        Some((mon_binary_str, vec!["-d".to_string(), path.to_string_lossy().to_string()]))
        //Some(path.clone().to_str().unwrap())
    } else if let Some(regex) = args.regex {
        Some((mon_binary_str, vec!["-r".to_string(), regex]))
        //Some(regex.clone().as_str())
    } else if let Some(regex) = args.grep {
        streaming_grep_mon("grep", "-Ex", regex);
        None
    } else if let Some(regex) = args.ripgrep {
        streaming_grep_mon("rg", "-x", regex);
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

fn _nonstreaming_grep_monitor(grep_cmd: &str, options: &str, regex: String) {
    //Run grep command
    let mut grep_cmd = Command::new(grep_cmd)
        .args([options, "-c", regex.as_str()]) //Just need a count of matching lines
        .stdin(Stdio::piped())
        .stdout(Stdio::piped()) 
        .stderr(Stdio::inherit())
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

fn streaming_grep_mon(grep_cmd: &str, options: &str, regex: String) {
    //Run grep command
    let mut grep_cmd = Command::new(grep_cmd)
        .args([options, "-n", regex.as_str()]) //Line numbers will be needed for this implementation 
        .stdin(Stdio::piped())
        .stdout(Stdio::piped()) 
        .stderr(Stdio::inherit())
        .spawn().expect("Command failed to execute");
    //Tee stdin into both child process and parent process (parent process only receives line numbers though)
    let mut child_stdin = grep_cmd.stdin.take().expect("Failed to open child stdin");
    let (reader, mut writer) = pipe().unwrap();
    thread::spawn(move || {
        let stdin_reader = io::stdin().lock();
        //let mut stdout_writer = io::stdout().lock(); - For no overhead streaming implementation
        let mut counter: u32 = 1;
        for line_res in stdin_reader.lines() {
            let line = line_res.expect("Error reading line of stdin");
            writeln!(child_stdin, "{}", line).expect("Failed to write to child stdin"); //Write line to child
            writer.write(&counter.to_le_bytes()).expect("Failed to write line number to parent"); //Write line number to parent
            counter += 1;
            //writeln!(stdout_writer, "{}", line).expect("Failed to write line to stdout"); - For no overhead streaming implementation
        }
    });
    //Receive output from child and ensure that child line numbers match expected (if not we have a non-match)
    let grep_stdout = grep_cmd.stdout.take().expect("Failed to capture stdout");
    let grep_reader = BufReader::new(grep_stdout);
    let mut counter_reader = BufReader::new(reader);
    let mut line_no_buf = [0u8; 4];
    let mut stdout_writer = io::stdout().lock();
    for line_res in grep_reader.lines() {
        let line = line_res.expect("Error reading line from child grep process");
        let (grep_line_no_str, _rest) = line.split_once(':').expect("Line missing colon");
        let grep_line_no = grep_line_no_str.parse::<u32>().expect("Line number unable to parse to u32");
        counter_reader.read_exact(&mut line_no_buf).expect("Ran out of line numbers before lines from grep process...");
        if grep_line_no != u32::from_le_bytes(line_no_buf) {
            panic!("Validation Failed\nIncriminating Line Number: {}", u32::from_le_bytes(line_no_buf));
        }
        let line_wo_line_no = line.chars().skip(2).collect::<String>();
        writeln!(stdout_writer, "{}", line_wo_line_no).expect("Failed to write line to stdout");
    }
    //Check to make sure that there are not still line numbers left in the pipe 
    match counter_reader.read_exact(&mut line_no_buf) {
        Ok(()) => panic!("Validation Failed\nIncriminating Line Number: {}", u32::from_le_bytes(line_no_buf)),
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {}, //This is the expected behavior of a valid match
        Err(e) => panic!("Non EOF error reading line number: {}", e)
    }

}