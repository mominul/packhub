#!/bin/bash

apt update
apt install sudo wget -y

echo
echo "Running the package key and repository setup script"

wget -qO- http://localhost:3000/sh/$DIST/github/mominul/pack-exp3 | sh
return_value=$?

if [ $return_value -ne 0 ]; then
    echo "The script failed with exit code $return_value"
    # Handle error case here
    exit $return_value
else
    echo "Package key and repository setup script ran successfully."
fi

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
else
    echo "Error: package not found." >&2
    exit 1
fi

# check if apt can install the package
apt_out=$(apt install openbangla-keyboard -y 2>&1)
apt_status=$?

# Print the output of the apt command
echo "$apt_out"

# Check if the apt command was successful
if [ $apt_status -ne 0 ]; then
    echo "Error: apt install command failed." >&2
    exit $apt_status
fi

echo "Package installed successfully."
exit 0
