use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicU32, Ordering};

const MON_BINARY: &str = "../streamonitor";
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

struct BenchMark {
    num: u32,
    cmd: String,
    typ: String,
}
impl BenchMark {
    fn new(cmd: String, typ: String) -> Self { Self { cmd, typ, num: INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst) } }
    fn bench(&self) -> f64 {
        let (_raw_stat, raw_time) = self.time_raw();
        let (_mon_stat, mon_time) = self.time_mon();
        let delta = mon_time - raw_time;
        println!(
            "Benchmark {}:\n Time Raw: {:?}, Time w/ Monitor:{:?}\nDelta: {:?}",
            self.num, raw_time, mon_time, delta
        );
        mon_time.as_secs_f64() / raw_time.as_secs_f64()
    }
    fn time_exec(full_cmd: String) -> (ExitStatus, Duration) {
        let timer = Instant::now();
        let exit_stat = Command::new("sh")
            .arg("-c").arg(full_cmd)
            .status().expect("Error running command...");
        (exit_stat, timer.elapsed())
    }
    fn time_raw(&self) -> (ExitStatus, Duration) { Self::time_exec(self.cmd.clone()) }
    fn time_mon(&self) -> (ExitStatus, Duration) { 
        Self::time_exec(format!("{} | {} -d {}", self.cmd, MON_BINARY, self.make_dfa().to_str().unwrap())) 
    }
    fn make_dfa(&self) -> PathBuf { todo!() }
}

fn main() {
    let commands = vec![("cmd1", "type1")];
    let mut ratios = Vec::new();
    for test in commands {
        let bench = BenchMark::new(String::from(test.0), String::from(test.1));
        ratios.push(bench.bench());
    }
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    println!("Average monitor time / raw time ratio: {:?}", avg_ratio);
}
