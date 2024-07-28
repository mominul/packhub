#!/bin/sh

echo "Welcome to package key and repository setup script for {{repo}}"
echo "This script will add the repository key and repository to your system."
echo "Please make sure you have sudo access to run this script."
echo
echo "Downloading and installing the repository key..."
wget -qO- http://localhost:3000/keys/packhub.gpg | sudo tee /etc/apt/keyrings/packhub.gpg > /dev/null
echo
echo "Adding the repository to your system..."
echo "deb [signed-by=/etc/apt/keyrings/packhub.gpg] http://localhost:3000/v1/apt/{{distro}}/github/{{owner}}/{{repo}} stable main" > /etc/apt/sources.list.d/{{repo}}.list
echo 
echo "Updating package lists..."
sudo apt-get update
