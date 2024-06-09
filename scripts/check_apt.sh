#!/bin/bash

echo "deb [trusted=yes] http://localhost:3000/apt/github/mominul/pack-exp3 stable main" > /etc/apt/sources.list.d/openbangla-keyboard.list

apt update

output=$(apt search openbangla 2>&1)
status=$?

# Print the output of the apt command
echo "$output"

# Check if the apt command was successful
if [ $status -ne 0 ]; then
    echo "Error: apt search command failed." >&2
    exit $status
fi

if echo "$output" | grep -q "openbangla-keyboard"; then
    echo
    echo "Package found successfully."
    exit 0
else
    echo "Error: package not found." >&2
    exit 1
fi
