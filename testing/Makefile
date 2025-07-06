.PHONY: dfa_builder

dfa_builder:
	cd ../regex-dfa-builder && mvn clean package
	cp ../regex-dfa-builder/target/dfa-builder.jar ./

clean:
	rm -rf dfa-cache/
	rm -f benchmark_results.csv