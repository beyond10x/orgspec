#!/bin/sh
# Hold the KPI catalogue: kpis/catalogue.yaml parses, follows kpis/README.md, and names real
# blocks and real events.
#
# Rules:
#   1. format is orgspec-kpis/1; every KPI has id, block, measures, over, unit, summary, how,
#      direction, gate, reads and status;
#   2. ids are snake_case and unique;
#   3. block is a domain of blocks/system.yaml (without `org.`, `shared` excluded) or one of the
#      planned blocks; every block of the system has at least 2 KPIs;
#   4. measures, over, unit, direction, status and each posture value come from their lists;
#      `over` is never a party, a membership or an assignment: nobody is scored;
#   5. a quality KPI is no gate;
#   6. every event in `reads` is declared in the `events:` section of a block; `reads` is empty
#      only for a KPI of status `checked`, which `orgspec check` counts from a design.
# Needs yq (mikefarah, v4).
#
# Usage: scripts/check-kpis.sh [FILE]
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
file="${1:-$root/kpis/catalogue.yaml}"
planned="initiative capability capacity reward review change"

command -v yq >/dev/null 2>&1 || { echo "check-kpis: yq is needed" >&2; exit 2; }
work="$(mktemp -d "${TMPDIR:-/tmp}/check-kpis.XXXXXX")"
trap 'rm -rf "$work"' EXIT
say() { echo "check-kpis: $*" >&2; echo x >>"$work/fail"; }

yq '.format' "$file" >"$work/format" 2>"$work/err" || { cat "$work/err" >&2; exit 1; }
[ "$(cat "$work/format")" = "orgspec-kpis/1" ] || say "format is not orgspec-kpis/1"

yq '.kpis[] | .id + " " + .block' "$file" >"$work/id-block"
yq '.kpis[] | .id + " " + (.measures // "-") + " " + (.over // "-") + " " + (.unit // "-") + " " + (.direction // "-") + " " + (.status // "-") + " " + (.gate | tag)' "$file" >"$work/fields"
yq '.kpis[] | .id + " " + (.measures // "-") + " " + (.gate // "-")' "$file" >"$work/gates"
yq '.kpis[] | select(.summary == null or .how == null or .reads == null) | .id' "$file" >"$work/missing"
yq '.kpis[] | .id + " " + (.postures.explore // "-") + " " + (.postures.scale // "-") + " " + (.postures.operate // "-") + " " + ((.postures // {}) | keys | join(","))' "$file" >"$work/postures"
yq '.kpis[] | .id + " " + (.status // "-") + " " + (.reads // [] | join(" "))' "$file" >"$work/reads"

while read -r id; do
  say "$id: summary, how or reads missing"
done <"$work/missing"

cut -d' ' -f1 "$work/id-block" | sort | uniq -d | while read -r id; do
  say "duplicate id $id"
done
cut -d' ' -f1 "$work/id-block" | grep -v -E '^[a-z][a-z0-9_]*$' | while read -r id; do
  say "id $id is not snake_case"
done

blocks="$(yq '.domains[]' "$root/blocks/system.yaml" | sed -n 's/^org\.//p' | grep -v '^shared$')"
while read -r id block; do
  case " $(echo $blocks) $planned " in
    *" $block "*) ;;
    *) say "$id: block $block is not a block" ;;
  esac
done <"$work/id-block"
for b in $blocks; do
  n="$(cut -d' ' -f2 "$work/id-block" | grep -c -x "$b" || true)"
  [ "$n" -ge 2 ] || say "block $b has $n KPIs, needs at least 2"
done

while read -r id measures over unit direction status gatetag; do
  case "$measures" in correctness|completeness|quality) ;; *) say "$id: measures $measures" ;; esac
  case "$over" in
    organization|unit|role|position|decision_class|policy|offering|relationship|goal|asset|budget|process|agreement|forum|channel|risk|obligation|design|type) ;;
    party|membership|assignment) say "$id: read over $over; a KPI is never read over a party" ;;
    *) say "$id: over $over" ;;
  esac
  case "$unit" in share|count|seconds|days|money|ratio|points) ;; *) say "$id: unit $unit" ;; esac
  case "$direction" in higher|lower) ;; *) say "$id: direction $direction" ;; esac
  case "$status" in checked|simulated|planned) ;; *) say "$id: status $status" ;; esac
  case "$gatetag" in '!!bool') ;; *) say "$id: gate is not a boolean" ;; esac
done <"$work/fields"

while read -r id measures gate; do
  if [ "$measures" = quality ] && [ "$gate" = true ]; then
    say "$id: a quality KPI is a gate"
  fi
done <"$work/gates"

while read -r id e s o keys; do
  for v in $e $s $o; do
    case "$v" in higher|lower|ignore|-) ;; *) say "$id: posture value $v" ;; esac
  done
  for k in $(echo "${keys:-}" | tr ',' ' '); do
    case "$k" in explore|scale|operate) ;; *) say "$id: posture $k" ;; esac
  done
done <"$work/postures"

for f in "$root"/blocks/*/*.yaml; do
  awk '/^events:/ {e=1; next} /^[a-z]/ {e=0} e && /^  - name: / {print $3}' "$f"
done | sort -u >"$work/events"
while read -r id status events; do
  if [ -z "${events:-}" ] && [ "$status" != checked ]; then
    say "$id: reads no event and is not of status checked"
  fi
  for event in ${events:-}; do
    grep -q -x "$event" "$work/events" || say "$id: reads $event, which is not a declared event"
  done
done <"$work/reads"

if [ -s "$work/fail" ]; then
  exit 1
fi
echo "check-kpis: $(wc -l <"$work/id-block" | tr -d ' ') KPIs ok"
