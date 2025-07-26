.PHONY: dfa_builder multi-monitor

multi-monitor:
	cd ../multi-monitor && make monitor && cargo build --release

dfa_builder:
	cd ../regex-dfa-builder && mvn clean package
	cp ../regex-dfa-builder/target/dfa-builder.jar ./

clean:
	rm -rf dfa-cache/
	rm -f benchmark_results.csv