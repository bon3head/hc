#!/usr/bin/env bash
# Prepare the local test environment.
set -e
cd "$(dirname "$0")/.."
mkdir -p .canary
date -u +%FT%TZ > .canary/I1
