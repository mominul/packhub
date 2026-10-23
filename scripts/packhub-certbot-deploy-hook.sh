#!/usr/bin/env bash
set -euo pipefail

# Runs only after a certificate was actually renewed.
docker kill -s HUP "${PACKHUB_APP_CONTAINER:-packhub_app}"
