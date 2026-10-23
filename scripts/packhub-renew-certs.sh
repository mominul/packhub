#!/usr/bin/env bash
set -euo pipefail

# Spread renewals across the hour so hosts do not hit the CA in lockstep.
sleep $((RANDOM % 3600))

# certbot reads the webroot path from each renewal config written by
# `certbot certonly --webroot ...`, so no flags are needed here.
exec certbot renew --quiet
