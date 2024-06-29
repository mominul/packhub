#!/bin/bash

apt update
apt install sudo wget -y

wget -qO- http://localhost:3000/sh/github/ubuntu/mominul/pack-exp2 | sh

output=$(apt search openbangla 2>&1)
status=$?

# Print the output of the apt command
echo "$output"

# Check if the apt command was successful
if [ $status -ne 0 ]; then
    echo "Error: apt search command failed." >&2
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
