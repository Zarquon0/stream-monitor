#!/bin/bash

# Usage: ./duplicate.sh input.txt

if [ $# -ne 1 ]; then
  echo "Usage: $0 input_file"
  exit 1
fi

input_file="$1"

# Store original contents
temp_file=$(mktemp)
cp "$input_file" "$temp_file"

# Truncate original file
> "$input_file"

# Append original contents 100,000 times
for i in $(seq 1 100000); do
  cat "$temp_file" >> "$input_file"
done

# Clean up
rm "$temp_file"