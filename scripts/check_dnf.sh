#!/bin/bash

wget -qO- http://localhost:3000/sh/yum/github/mominul/pack-exp3 | sh
output=$(yes | dnf search openbangla 2>&1)
status=$?

# Print the output of the dnf command
echo "$output"

# Check if the dnf command was successful
if [ $status -ne 0 ]; then
    echo "Error: dnf search command failed." >&2
    exit $status
fi

if echo "$output" | grep -q "openbangla-keyboard"; then
    echo
    echo "Package found successfully."
else
    echo "Error: package not found." >&2
    exit 1
fi

# check if dnf can install the package
dnf_out=$(yes | dnf install openbangla-keyboard 2>&1)
dnf_status=$?

# Print the output of the dnf command
echo "$dnf_out"

# Check if the dnf command was successful
if [ $dnf_status -ne 0 ]; then
    echo "Error: dnf install command failed." >&2
    exit $dnf_status
fi

echo "Package installed successfully."
exit 0
