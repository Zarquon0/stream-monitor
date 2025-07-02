use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::process::{Command, ExitStatus, Stdio, exit};
use std::sync::atomic::{AtomicU32, Ordering};
use std::io::{Result, Error, ErrorKind};
use std::fs::{create_dir, File};
use std::env::consts::OS;
use csv::ReaderBuilder;

const MON_BINARY: &str = "../target/release/monitor";
const DFA_BUILDER_JAR: &str = "dfa-builder.jar";
const DFA_CACHE: &str = "dfa-cache";
const BENCHMARKS_CSV: &str = "benchmarks.csv";
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

struct BenchMark {
    num: u32,
    cmd: String,
    typ: String,
    proj_root: PathBuf,
}
impl BenchMark {
    fn new(cmd: String, typ: String) -> Self { 
        Self { 
            cmd, typ, 
            num: INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst), 
            proj_root: proj_root() 
        } 
    }
    fn bench(&self) -> Option<f64> {
        let raw_time = self.time_raw()?;
        let (mon_stat, mon_time) =  self.time_mon()?;
        if !mon_stat.success() {
            eprintln!("\nBenchmark {} Failed! (See above ^^^)\nCommand: {}\nType: {}", self.num, self.cmd, self.typ);
            return None
        }
        let delta = mon_time - raw_time;
        println!(
            "\nBenchmark {} (Command: {}, Type: {}):\n Time Raw: {:?}, Time w/ Monitor:{:?}\nDelta: {:?}",
            self.num, self.cmd, self.typ, raw_time, mon_time, delta
        );
        Some(mon_time.as_secs_f64() / raw_time.as_secs_f64())
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
        let full_res = Self::handle_test_res(test_res, "Running command raw failed")?;
        Some(full_res.1)
    }
    fn time_mon(&self) -> Option<(ExitStatus, Duration)> { 
        let dfa_path = Self::handle_test_res(self.make_dfa(), "DFA creation failed")?;
        let test_res = Self::time_exec(format!(
            "{} | {} -d {}", 
            self.cmd, 
            self.proj_root.join(MON_BINARY).to_str().unwrap(),
            dfa_path.to_str().unwrap()));
        Self::handle_test_res(test_res, "Running command with monitor failed")
    }
    fn make_dfa(&self) -> Result<PathBuf> { 
        let builder_out = Command::new("java")
            .args(["-jar", self.proj_root.join(DFA_BUILDER_JAR).to_str().unwrap(), self.typ.as_str()])
            .current_dir(self.proj_root.join(DFA_CACHE))
            .output()?;
        if !builder_out.status.success() { return Err(Error::new(
            ErrorKind::InvalidData, 
            format!("DFA builder errored trying to build {}", self.typ)
        )) }
        Ok(self.proj_root.join(DFA_CACHE).join(PathBuf::from(String::from_utf8_lossy(&builder_out.stdout).trim())))
    }
    fn handle_test_res<T>(test_res: Result<T>, msg: &str) -> Option<T> {
        match test_res {
            Ok(res) => Some(res),
            Err(e) => {
                eprintln!("{}: {:?}", msg, e);
                None
            }
        }
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

fn main() {
    //OS check - this script should not be run from on a non-linux OS
    if !(OS == "linux") { 
        eprintln!("!! This script runs linux commands, so it must be run on a linux OS !!");
        exit(1);
    }
    //Read command-type pairs in from CSV
    let commands = read_commands().expect("Error reading commands from CSV");
    //Create cache directory if it doesn't exist
    let cache_dir = proj_root().join(DFA_CACHE);
    if !cache_dir.exists() { create_dir(cache_dir).expect("Failed to create dfa cache dir"); }
    //Run bench marks and collect ratios
    let mut ratios = Vec::new();
    for test in commands {
        let bench = BenchMark::new(test.0, test.1);
        if let Some(ratio) = bench.bench() { ratios.push(ratio); }
    }
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    println!("Average monitor time / raw time ratio: {:?}\n", avg_ratio);
}
