#!/bin/bash

apt update
apt install sudo wget -y

echo
echo "Running the package key and repository setup script"
wget -qO- http://host.docker.internal:3000/sh/github/ubuntu/mominul/pack-exp3 | sh
return_value=$?

if [ $return_value -ne 0 ]; then
    echo "The script failed with exit code $return_value"
    # Handle error case here
else
    echo "The script completed successfully"
fi
echo "Package key and repository setup script ran successfully."

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
