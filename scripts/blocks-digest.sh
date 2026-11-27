#!/bin/sh
# blocks-digest.sh: print one SHA-256 over the building blocks, so that a copy of blocks/ in
# another repository (an organization type's extension system) can be proved unchanged.
#
#   scripts/blocks-digest.sh [--path DIR]     DIR defaults to blocks/ of this repository
#
# The digest covers system.yaml, ess-inputs.yaml and every file ess-inputs.yaml lists under
# `specification:`, each with its path, in the order of the list. Scenarios are not part of it:
# a copy may leave them out. A file the list names and the directory lacks is an error.

set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
path=$root/blocks

while [ $# -gt 0 ]; do
  case "$1" in
    --path) [ $# -ge 2 ] || { echo "blocks-digest: --path needs a directory" >&2; exit 2; }
            path=$2; shift 2 ;;
    *) echo "blocks-digest: unknown argument: $1" >&2; exit 2 ;;
  esac
done

[ -f "$path/ess-inputs.yaml" ] || { echo "blocks-digest: no ess-inputs.yaml in $path" >&2; exit 2; }
cd "$path"

files=$(awk '/^specification:/ { on = 1; next } /^[a-z_]+:/ { on = 0 } on && /^  - / { sub(/^  - /, ""); print }' ess-inputs.yaml)

for f in $files; do
  [ -f "$f" ] || { echo "blocks-digest: listed and missing: $f" >&2; exit 2; }
done

for f in $files; do
  printf '%s %s\n' "$(sha256sum < "$f" | cut -d' ' -f1)" "$f"
done | sha256sum | cut -d' ' -f1
