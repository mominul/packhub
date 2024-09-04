#!/bin/bash

sh -c 'echo -e "[obk]\nname=OpenBangla Keyboard\nbaseurl=http://localhost:3000/v1/rpm/github/mominul/pack-exp2\nenabled=1\ngpgcheck=0\nrepo_gpgcheck=1\ngpgkey=http://localhost:3000/keys/packhub.asc" > /etc/zypp/repos.d/obk.repo'

echo 'a' | zypper refresh

output=$(yes | zypper search openbangla 2>&1)
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
