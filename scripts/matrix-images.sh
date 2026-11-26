#!/usr/bin/env bash
# Build minimal Ubuntu rootfs images locally (Docker Hub is not reachable here) and import them.
set -euo pipefail
apt-get install -y -qq debootstrap >/dev/null
build() {
  local suite=$1 dir=/home/claude/matrix/rootfs-$1
  [[ -d $dir/usr ]] || debootstrap --variant=minbase --include=git,openssh-client,python3,ca-certificates "$suite" "$dir" http://archive.ubuntu.com/ubuntu >"/home/claude/matrix/$suite.log" 2>&1
  tar -C "$dir" -c . | docker import - "hc-matrix:$suite" >/dev/null
  echo "$suite ok"
}
build focal & build jammy & build noble & wait
