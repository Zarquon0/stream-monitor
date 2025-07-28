use std::{error, result};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::process::{Command, ExitStatus, Stdio, exit};
use std::sync::atomic::{AtomicU32, Ordering};
use std::io::{Result, Error, ErrorKind};
use std::fs::{create_dir, File};
use std::env::consts::OS;
use clap::Parser;
use csv::{ReaderBuilder, Writer};
use monitor::Dfa;

const MON_BINARY: &str = "../target/release/monitor";
const MULTIMON_BINARY: &str = "../target/release/multi-monitor";
const DFA_BUILDER_JAR: &str = "dfa-builder.jar";
const DFA_CACHE: &str = "dfa-cache";
const BENCHMARKS_CSV: &str = "benchmarks.csv";
const RESULTS_CSV: &str = "benchmark_results.csv";
const COMP_RESULTS_CSV: &str = "comp_benchmark_results.csv";
const TRIALS: u8 = 5;
const BAD_CHARS: [char; 2] = ['<','>'];
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

#[derive(Debug)]
struct BenchRes {
    pub num: u32,
    pub cmd: String,
    pub typ: String,
    pub raw_time: Duration,
    pub mon_time: Duration,
}

struct BenchMark {
    num: u32,
    cmd: String,
    typ: String,
    proj_root: PathBuf,
}
impl BenchMark { //Core functionality for speed benchmarking w/ DFAs
    fn new(cmd: String, typ: String) -> Self { 
        Self { 
            cmd, typ, 
            num: INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst), 
            proj_root: proj_root() 
        } 
    }
    fn bench(&self) -> Option<BenchRes> {
        let mut raw_times = Vec::new();
        let mut mon_times = Vec::new();
        for _ in 0..TRIALS {
            raw_times.push(self.time_raw()?);
            let (mon_stat, mon_time) = self.time_mon()?;
            if !mon_stat.success() {
                eprintln!("\nBenchmark {} Failed! (See above ^^^)\nCommand: {}\nType: {}", self.num, self.cmd, self.typ);
                return None
            }
            mon_times.push(mon_time);
        }
        let avg_raw_time = raw_times.iter().sum::<Duration>() / TRIALS as u32;
        let avg_mon_time = mon_times.iter().sum::<Duration>() / TRIALS as u32;
        Some(BenchRes{ num: self.num, cmd: self.cmd.clone(), typ: self.typ.clone(), raw_time: avg_raw_time, mon_time: avg_mon_time })
    }
    fn time_exec(full_cmd: String) -> Result<(ExitStatus, Duration)> {
        let timer = Instant::now();
        let exit_stat = Command::new("sh")
            .stdout(Stdio::null()) // discard stdout
            .arg("-c").arg(full_cmd)
            .status()?;
        Ok((exit_stat, timer.elapsed()))
    }
    fn time_raw(&self) -> Option<Duration> { 
        let test_res = Self::time_exec(self.cmd.clone());
        let full_res = self.handle_test_res(test_res, "Running command raw failed")?;
        Some(full_res.1)
    }
    fn time_mon(&self) -> Option<(ExitStatus, Duration)> { 
        let dfa_path = self.handle_test_res(self.make_dfa(), "DFA creation failed")?;
        let test_res = Self::time_exec(format!(
            "{} | {} -d {}", 
            self.cmd, 
            self.proj_root.join(MON_BINARY).to_str().unwrap(),
            dfa_path.to_str().unwrap()));
        self.handle_test_res(test_res, "Running command with monitor failed")
    }
    fn make_dfa(&self) -> Result<PathBuf> {
        //Build JSON DFA representation out of type (regular expression)
        let escaped_regex = Self::escape_bad_chars(&self.typ);
        let builder_out = Command::new("java")
            .args(["-jar", self.proj_root.join(DFA_BUILDER_JAR).to_str().unwrap(), escaped_regex.as_str()])
            .current_dir(self.proj_root.join(DFA_CACHE))
            .output()?;
        if !builder_out.status.success() || !builder_out.stderr.is_empty() { return Err(Error::new(
            ErrorKind::InvalidData, 
            format!("DFA builder errored\nTo debug, try running java -jar dfa-builder.jar \"{}\"", escaped_regex)
        )) }
        let json_dfa_path = self.proj_root.join(DFA_CACHE).join(PathBuf::from(String::from_utf8_lossy(&builder_out.stdout).trim()));
        //Ok(json_dfa_path)
        //Deserialize into Dfa from JSON, then serialize into binary form (for quicker deserialization)
        Ok(Dfa::deserialize_from_json(json_dfa_path).serialize())
    }
    fn escape_bad_chars(regex: &String) -> String {
        let mut escaped = String::new();
        for c in regex.clone().chars() {
            if BAD_CHARS.contains(&c) {
                escaped.push('\\'); // Escape with backslash
            }
            escaped.push(c);
        }
        escaped
    }
    fn handle_test_res<T>(&self, test_res: Result<T>, msg: &str) -> Option<T> {
        match test_res {
            Ok(res) => Some(res),
            Err(e) => {
                eprintln!("\nBenchmark {} failed:\n{}:\n{}\n", self.num, msg, e.to_string());
                None
            }
        }
    }
}


#[derive(Debug)]
enum Mode { Dfa, Regex, Grep, Ripgrep }

#[derive(Debug)]
struct ModeDurations(Duration, Duration, Duration, Duration);
#[derive(Debug)]
struct CompBenchRes {
    pub num: u32,
    pub cmd: String,
    pub typ: String,
    pub mode_times: ModeDurations,
}

impl BenchMark { //Functionality for implementation comparative benchmarking
    fn time_multi_mode(&self, mode: &Mode) -> Option<(ExitStatus, Duration)> {
        let opt_string = match mode {
            Mode::Dfa => {
                let dfa_path = self.handle_test_res(self.make_dfa(), "DFA creation failed")?;
                format!("-d {}", dfa_path.to_str().unwrap())
            }
            Mode::Regex => format!("-r \"{}\"", self.typ),
            Mode::Grep => format!("-g \"{}\"", self.typ),
            Mode::Ripgrep => format!("-R \"{}\"", self.typ),
        };
        let test_res = Self::time_exec(format!(
            "{} | {} {}", 
            self.cmd, 
            self.proj_root.join(MULTIMON_BINARY).to_str().unwrap(),
            opt_string
        ));
        self.handle_test_res(test_res, format!("Running command with multi-monitor mode {:?} failed", mode).as_str())
    }
    fn comparative_bench(&self) -> Option<CompBenchRes> {
        let mut mode_times: Vec<(Mode, Vec<Duration>)> = vec![
            (Mode::Dfa, Vec::new()),
            (Mode::Regex, Vec::new()),
            (Mode::Grep, Vec::new()),
            (Mode::Ripgrep, Vec::new()),
        ];
        for _ in 0..TRIALS {
            for mode_time in &mut mode_times {
                let (stat, dur) = self.time_multi_mode(&mode_time.0)?;
                if !stat.success() {
                    eprintln!("\nBenchmark {} Failed! (See above ^^^)\nCommand: {}\nType: {}", self.num, self.cmd, self.typ);
                    return None
                }
                mode_time.1.push(dur);
            }
        }
        let mode_avg_times: Vec<(Mode, Duration)> = mode_times.drain(..).map(
            |mode_time| (
                mode_time.0,
                mode_time.1.iter().sum::<Duration>() / TRIALS as u32
            )
        ).collect();
        let mode_durations = ModeDurations(
            mode_avg_times[0].1,
            mode_avg_times[1].1,
            mode_avg_times[2].1,
            mode_avg_times[3].1,
        );
        Some(CompBenchRes { num: self.num, cmd: self.cmd.clone(), typ: self.typ.clone(), mode_times: mode_durations })
    }
}

fn proj_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_commands() -> Result<Vec<(String, String)>> {
    let file = File::open(proj_root().join(BENCHMARKS_CSV))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    let mut result = Vec::new();
    for record in rdr.records() {
        let record = record?;
        if record.len() != 2 { 
            return Err(Error::other(format!("Invalid CSV record: {:?}", record))); 
        }
        let command = record[0].trim().to_string();
        let regex = record[1].trim().to_string();
        result.push((command, regex));
    }
    Ok(result)
}

fn export_speed_results(results: Vec<BenchRes>) -> result::Result<(), Box<dyn error::Error>> {
    let file = File::create(proj_root().join(RESULTS_CSV))?;
    let mut writer = Writer::from_writer(file);
    writer.write_record(&["Number", "Command", "Raw Time (ms)", "With Monitor Time (ms)", "Type"])?;
    for res in results {
        writer.write_record(&[
            res.num.to_string(),
            res.cmd,
            duration_to_ms(&res.raw_time).to_string(),
            duration_to_ms(&res.mon_time).to_string(),
            res.typ, // Automatically quoted if it contains commas
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn export_comp_results(results: Vec<CompBenchRes>) -> result::Result<(), Box<dyn error::Error>> {
    let file = File::create(proj_root().join(COMP_RESULTS_CSV))?;
    let mut writer = Writer::from_writer(file);
    writer.write_record(&["Number", "Command", "DFA Time (ms)", "Regex Time (ms)", "Grep Time (ms)", "Ripgrep Time (ms)", "Type"])?;
    for res in results {
        writer.write_record(&[
            res.num.to_string(),
            res.cmd,
            duration_to_ms(&res.mode_times.0).to_string(),
            duration_to_ms(&res.mode_times.1).to_string(),
            duration_to_ms(&res.mode_times.2).to_string(),
            duration_to_ms(&res.mode_times.3).to_string(),
            res.typ, // Automatically quoted if it contains commas
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn duration_to_ms(dur: &Duration) -> f64 {
    dur.as_secs() as f64 * 1000.0 + f64::from(dur.subsec_nanos()) / 1_000_000.0
}

fn speed_bench(bmarks: Vec<BenchMark>) {
    let mut ratios = Vec::new();
    let mut times = Vec::new();
    for bench in bmarks {
        if let Some(br) = bench.bench() {
            let delta =  duration_to_ms(&br.mon_time) - duration_to_ms(&br.raw_time);
            println!(
                "\nBenchmark {} (Command: {}, Type: {}):\n Time Raw: {:?}, Time w/ Monitor:{:?}\nDelta: {:?} ms",
                br.num, br.cmd, br.typ, br.raw_time, br.mon_time, delta
            );
            let ratio = br.mon_time.as_secs_f64() / br.raw_time.as_secs_f64();
            ratios.push(ratio); 
            times.push(br);
        }
    }
    //Print average time ratio
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    println!("\nAverage monitor time / raw time ratio: {:?}\n", avg_ratio);
    //Save Results to CSV
    export_speed_results(times).expect(format!("Error saving results to {}", RESULTS_CSV).as_str());
}

fn comp_bench(bmarks: Vec<BenchMark>) {
    let mut times = Vec::new();
    for bench in bmarks {
        if let Some(br) = bench.comparative_bench() {
            println!(
                "\nBenchmark {} (Command: {}, Type: {}):\nDFA\t\tRegex\t\tGrep\t\tRipgrep\n{:?}\t{:?}\t{:?}\t{:?}",
                br.num, br.cmd, br.typ, br.mode_times.0, br.mode_times.1, br.mode_times.2, br.mode_times.3
            );
            times.push(br);
        }
    }
    export_comp_results(times).expect(format!("Error saving results to {}", COMP_RESULTS_CSV).as_str());
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Specifies that implementation comparative should be performed instead of standard testing
    #[arg(short, default_value_t = false)]
    comparative_testing: bool,
}

fn main() {
    //OS check - this script should not be run from on a non-linux OS
    if !(OS == "linux") { 
        eprintln!("!! This script runs linux commands, so it must be run on a linux OS !!");
        exit(1);
    }
    //Read command-type pairs in from CSV
    let mut commands = read_commands().expect("Error reading commands from CSV");
    //Create cache directory if it doesn't exist
    let cache_dir = proj_root().join(DFA_CACHE);
    if !cache_dir.exists() { create_dir(cache_dir).expect("Failed to create dfa cache dir"); }
    //Clean DFA's built-in cache of DFAs 
    Dfa::clean_cache();
    //Create benchmarks from commands vector
    let bmarks = commands
        .drain(..)
        .map(|(cmd, typ)| BenchMark::new(cmd, typ))
        .collect::<Vec<BenchMark>>();
    //Run desired benchmarking
    let args = Args::parse();
    if args.comparative_testing { comp_bench(bmarks); }
    else { speed_bench(bmarks); }
}
