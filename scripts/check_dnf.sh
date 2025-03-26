#!/bin/bash

sh -c 'echo -e "[obk]\nname=OpenBangla Keyboard\nbaseurl=http://localhost:3000/v1/rpm/github/mominul/pack-exp3\nenabled=1\ngpgcheck=0\nrepo_gpgcheck=1\ngpgkey=http://localhost:3000/v1/keys/packhub.asc" > /etc/yum.repos.d/obk.repo'

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
    exit 0
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
