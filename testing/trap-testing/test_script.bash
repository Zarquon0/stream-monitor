#!/bin/bash
source ../../monitor/monitor_trap.sh

( # Should work in a subshell!
    echo "abcdB1" | ../../target/debug/monitor -t -d ../../json-to-dfa/serialized_example_dfa.bc
)