#!/bin/sh
# check-broken.sh: every design under organizations/broken/ must be refused by `orgspec check`
# for the rule its file is named after, and for no other gated rule.
#
#   scripts/check-broken.sh [--bin PATH]      PATH defaults to target/debug/orgspec
#
# organizations/broken/<rule_id>.yaml is a design that is valid except for one thing. The check
# proves that a rule can refuse: a rule no broken design names is listed at the end, not failed.
#
# Exit 0: every broken design is refused for its rule alone. Exit 1 otherwise.

set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
bin=target/debug/orgspec

while [ $# -gt 0 ]; do
  case "$1" in
    --bin) [ $# -ge 2 ] || { echo "check-broken: --bin needs a path" >&2; exit 2; }
           bin=$2; shift 2 ;;
    *) echo "check-broken: unknown argument: $1" >&2; exit 2 ;;
  esac
done

[ -x "$bin" ] || { echo "check-broken: no checker at $bin; run: cargo build -p orgspec" >&2; exit 2; }
command -v jq >/dev/null || { echo "check-broken: jq is missing" >&2; exit 2; }

n=0
bad=0
for f in organizations/broken/*.yaml; do
  [ -f "$f" ] || { echo "check-broken: no design under organizations/broken/" >&2; exit 1; }
  rule=$(basename "$f" .yaml)
  n=$((n + 1))
  status=0
  out=$("$bin" check --json "$f" 2>&1) || status=$?
  if [ "$status" -ne 1 ]; then
    echo "check-broken: $f: exit $status, wanted 1"
    bad=$((bad + 1))
    continue
  fi
  got=$(printf '%s' "$out" | jq -r '[.[0].violations[] | select(.gate) | .rule] | unique | join(" ")' 2>/dev/null) || got="unreadable output"
  if [ "$got" != "$rule" ]; then
    echo "check-broken: $f: refused for [$got], wanted [$rule]"
    bad=$((bad + 1))
  fi
done

if [ "$bad" -gt 0 ]; then
  echo "check-broken: $bad of $n broken design(s) not refused as named"
  exit 1
fi
echo "check-broken: $n broken design(s), each refused for the rule it is named after"
