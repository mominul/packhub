#!/bin/bash

yes | dnf install wget sudo

echo
echo "Running the package key and repository setup script"

wget -qO- http://localhost:3000/sh/yum/github/mominul/pack-exp2 | sh
return_value=$?
if [ $return_value -ne 0 ]; then
    echo "The script failed with exit code $return_value"
    # Handle error case here
    exit $return_value
else
    echo "Package key and repository setup script ran successfully."
fi

output=$(yes | dnf search openbangla 2>&1)
status=$?

# Print the output of the dnf command
echo "$output"

# Check if the dnf command was successful
if [ $status -ne 0 ]; then
    echo "Error: dnf search command failed." >&2
    exit $status
fi

# Check if `fcitx-openbangla` is in the output
if echo "$output" | grep -q "fcitx-openbangla"; then
    echo
    echo "Package fcitx-openbangla found."
else
    echo "Error: fcitx-openbangla not found." >&2
    exit 1
fi

# Check if `ibus-openbangla` is in the output
if echo "$output" | grep -q "ibus-openbangla"; then
    echo
    echo "Package ibus-openbangla found."
else
    echo "Error: ibus-openbangla not found." >&2
    exit 1
fi
