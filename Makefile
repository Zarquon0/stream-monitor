ROOT_DIR := $(shell pwd)

.PHONY: debug release perf_test

# cargo build --manifest-path $(ROOT_DIR)/monitor/Cargo.toml
debug:
	cargo build -p monitor 
	cp $(ROOT_DIR)/target/debug/monitor $(ROOT_DIR)/streamonitor

# Must be run with sudo
release:
	cargo build --release -p monitor;
	cp $(ROOT_DIR)/target/release/monitor $(ROOT_DIR)/streamonitor

perf_test:
	cd regex-dfa-builder && mvn clean package
	cp regex-dfa-builder/target/dfa-builder.jar testing/
	cp $(ROOT_DIR)/target/release/monitor $(ROOT_DIR)/streamonitor