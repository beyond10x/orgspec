#!/bin/sh
# Tests of scripts/planning-only.sh: both sides of the path list.

set -u

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sut=$here/planning-only.sh
fail=0
cases=0

# expect <exit> <path>... : feed the paths and compare the exit code
expect() {
  want=$1; shift
  cases=$((cases + 1))
  printf '%s\n' "$@" | "$sut" >/dev/null
  got=$?
  if [ "$got" != "$want" ]; then
    echo "planning-only.test: FAIL: exit $got, wanted $want, for: $*" >&2
    fail=1
  fi
}

expect 0 README.md
expect 0 design/building-blocks.md design/organization-types.md CHANGELOG.md
expect 0 .engineering/planning/story/one.md .engineering/project.yaml
expect 0 kpis/README.md AGENTS.md
expect 1 blocks/4-authority/decision.yaml
expect 1 README.md crates/orgspec/src/check.rs
expect 1 design/building-blocks.md types/regulated.yaml
expect 1 organizations/bakery-cooperative.yaml
expect 1 kpis/catalogue.yaml
expect 1 .github/workflows/gate.yml
expect 1 Taskfile.yml
expect 1 scripts/planning-only.sh
expect 1 ""

cases=$((cases + 1))
"$sut" extra </dev/null >/dev/null 2>&1
got=$?
[ "$got" = 2 ] || { echo "planning-only.test: FAIL: an argument gave exit $got, wanted 2" >&2; fail=1; }

[ "$fail" = 0 ] && echo "planning-only.test: $cases cases pass"
exit "$fail"
