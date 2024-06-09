#!/bin/bash

sh -c 'echo -e "[obk]\nname=OpenBangla Keyboard\nbaseurl=http://localhost:3000/rpm/github/OpenBangla/OpenBangla-Keyboard/\nenabled=1\ngpgcheck=0" > /etc/yum.repos.d/obk.repo'

output=$(dnf search openbangla 2>&1)
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
    exit 0
else
    echo "Error: package not found." >&2
    exit 1
fi
