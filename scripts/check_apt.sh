#!/bin/bash

apt update
apt install sudo wget -y

echo
echo "Running the package key and repository setup script"

# Save the script first for debugging
SCRIPT_URL="http://localhost:3000/sh/github/ubuntu/mominul/pack-exp3"
SCRIPT_LOCAL_PATH="/tmp/pack-exp3.sh"

# Try downloading the script
wget -qO "$SCRIPT_LOCAL_PATH" "$SCRIPT_URL"
return_value=$?

if [ $return_value -ne 0 ]; then
    echo "Failed to download the script with exit code $return_value"
    echo "Make sure the URL is correct and the host server is reachable."
    # Handle error case here
    exit $return_value
else
    echo "Script downloaded successfully to $SCRIPT_LOCAL_PATH"
fi

# Output the downloaded script for debugging
echo "Downloaded script content:"
cat "$SCRIPT_LOCAL_PATH"

# Run the script
bash "$SCRIPT_LOCAL_PATH"
return_value=$?

if [ $return_value -ne 0 ]; then
    echo "The script failed with exit code $return_value"
    # Handle error case here
    exit $return_value
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
