# stream-monitor
A monitor for type checking the output of commands in shell scripts. With a streamed input and access to a specially serialized DFA, efficiently runs a line by line check of the input over the DFA, panicking upon a failed match and streaming each line along upon a successful one.

## Dependencies
For building and running the stream monitor application, you just need <b>Rust</b>. I'm not sure what the minimum version of Rust you could use would be, but I built this project with `1.87.0`. See [Rust's Website](https://www.rust-lang.org/tools/install) for installation instructions.

For running the testing crate, you will need to be using a <em>linux machine</em> and the following packages/tools used to benchmark the project:
```bash
apt install acpi iproute2 net-tools iptables
```
You will also need <b>Java</b> (11 or higher) for the testing crate.

The testing crate requires Java because it relies upon a Java tool to convert regular expressions into DFAs; if, for whatever reason, you would like to work on/rebuild this tool, you will also need <b>Maven</b> ([installation instructions here](https://maven.apache.org/install.html)).

Lastly, if you want to use the tool that turns JSON specified DFAs into serialized Rust DFA objects (what the monitor directly operates with), you will want to
```bash
cargo install rust-script
```

## Docker
You could also skip the dependencies with the handy-dandy Dockerfile provided. I use <b>Docker Compose</b> because I think it makes building containers more ergonomic and bearable, so the project is set up to the plug-in ([installation instructions for the uninitiated](https://docs.docker.com/compose/install/)). Building and running are done by:
```bash
# cd stream-monitor
docker-compose build
docker-compose up -d
docker attach monitor-box # This is the conatiner's name
```
Container removal is done via:
```bash
docker-compose down
```
Important Note: This container mounts the project directory instead of copying it. The benefits of this are that you can make tweaks to the code on your local machine and they will be immediately reflected on the container. The downsides of this are that if your host OS is signficantly different from Ubuntu (which it probably is, provided you are using the container in the first place), you will need to rebuild binaries every time you switch between running them on your host and runnning them on the container. I think the tradeoff is worth it, but if you don't, you could always remove the volume mount from the `compose.yaml` and add an `ADD` directive to the Dockerfile.

## Running the Monitor
You can either
```bash
# Run through cargo!
# cd monitor
cargo run -- -h
```
or
```bash
# Run the binary!
# cd stream-monitor
make debug # or release, with sudo
chmod +x streamonitor
./streamonitor -h
```
Feeling brave? Let's try a full example!
```bash
# cd stream-monitor
echo "abcdA234" > test.txt
cd json-to-dfa
chmod +x parse_dfa.rs
./parse_dfa.rs -o ../ example_dfa.json
cd ..
make debug
./streamonitor -d sdfa-<hash_stuff>.bc test.txt # Just use tab completion after `sdfa` - the name is, unfortunately, non-deterministic *ugh*
```
If you see a bunch of "Next State: (<num>)" prints followed by the contents of `test.txt`, you've got the monitor and the json parser script set up properly!

## Running the Testing Harness
If you're in the container (or are using a linux machine with the proper dependencies installed), it should be as simple as
```bash
# cd testing
cargo run
```
If you'd like the view the results, look in `benchmark_results.csv` and if you'd like to change the benchmark that get run, you can do that in `benchmarks.csv`.

NOTE: Seeing panics or other errors while testing? The current set of benchmarks in `benchmarks.csv` is under development and many will not pass the monitor.
