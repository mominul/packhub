#!/bin/sh

echo "Welcome to package key and repository setup script for {{repo}}"
echo "This script will add the repository key and repository to your system."
echo "Please make sure you have sudo access to run this script."

echo -e "[{{repo_name()}}]\nname={{name()}}\nbaseurl={{base_url()}}\nenabled=1\ngpgcheck=0\nrepo_gpgcheck=1\ngpgkey={{host}}/v1/keys/packhub.asc" | sudo tee /etc/{{mgr}}/{{repo_name()}}.repo > /dev/null

echo
echo "Repository has been added to your system."
echo "Please update your package lists to start using the repository."
echo "Use 'dnf update' or 'yum update' or 'zypper refresh' depending on your package manager."
