#!/bin/sh
# Hold the layering of an ESS system: every dependency between domains points down.
#
# A domain's layer is the number that prefixes its directory (0-foundation, 1-party, ...,
# 12-evaluation). A dependency of domain A on domain B is any of:
#   - a declared type of B used anywhere in A: an entity field, a command input, an event, error
#     or view field, or a type A declares (nested in structs, unions, lists, optionals);
#   - the identity type of an entity of B (an identifier in the shared domain counts as a
#     dependency on the domain whose entity it identifies);
#   - a relation of an entity of A whose target is an entity of B.
# Rules:
#   1. no dependency points to a higher layer;
#   2. dependencies inside one layer form no cycle;
#   3. every domain of the system lives in a numbered layer directory.
# Read from `ess specify compile` output.
#
# Usage: scripts/check-layers.sh [--path DIR] [--graph]
#        scripts/check-layers.sh --negative   (each fixture under scripts/fixtures/layers must fail)
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
path="$root/blocks"
graph=0

while [ $# -gt 0 ]; do
  case "$1" in
    --path) path="$2"; shift 2 ;;
    --graph) graph=1; shift ;;
    --negative)
      bad=0
      for fixture in "$root"/scripts/fixtures/layers/*/; do
        name="$(basename "$fixture")"
        "$0" --path "$fixture" > /dev/null 2>&1 && rc=0 || rc=$?
        if [ "$rc" != 1 ]; then
          echo "negative: fixture $name exited $rc, it must fail the layer check (exit 1)" >&2
          bad=1
        else
          echo "negative: fixture $name refused"
        fi
      done
      exit "$bad"
      ;;
    *) echo "usage: $0 [--path DIR] [--graph] | --negative" >&2; exit 2 ;;
  esac
done

work="$root/target/check-layers/$(printf '%s' "$path" | cksum | cut -d' ' -f1)"
mkdir -p "$work"

ess specify compile --path "$path" --format json > "$work/ir.json" 2> "$work/compile.err" || {
  cat "$work/compile.err" >&2
  exit 2
}

# domain -> layer, from the directory of each domain file in the specification list of ess-inputs.yaml
: > "$work/layers.txt"
awk '/^specification:/ { on = 1; next } /^[a-z_]+:/ { on = 0 } on && /^  - / { sub(/^  - /, ""); print }' "$path/ess-inputs.yaml" | while read -r file; do
  case "$file" in
    */*.yaml) ;;
    *) continue ;;
  esac
  domain="$(sed -n 's/^domain: *//p' "$path/$file" | head -n 1)"
  [ -n "$domain" ] || continue
  dir="${file%%/*}"
  case "$dir" in
    [0-9]-*|[0-9][0-9]-*) printf '%s %s\n' "$domain" "${dir%%-*}" >> "$work/layers.txt" ;;
    *) printf '%s none\n' "$domain" >> "$work/layers.txt" ;;
  esac
done

jq -r '
  (.entities | to_entries | map({key: .value.identity.type_ref.name, value: .value.domain})
    | map(select(.key != null)) | from_entries) as $identity
  | ((.types // {}) | to_entries
      | map({key: .key, value: [.value | .. | objects | select(.kind? == "declared") | .name]})
      | from_entries) as $direct
  | def closure:
      . as $s
      | ([$s[], ($s[] | ($direct[.] // [])[])] | unique) as $n
      | if ($n | length) == ($s | length) then $s else ($n | closure) end;
  def prefix: split(".")[0:2] | join(".");
  def owner: $identity[.] // prefix;
  (
    ("entities", "commands", "events", "views", "errors", "types") as $c
    | (.[$c] // {}) | to_entries[]
    | .key as $k | .value as $v
    | ($v.domain? // ($k | prefix)) as $from
    | ([$v | .. | objects | select(.kind? == "declared") | .name] | unique | closure)[]
    | owner as $to
    | select($to != $from)
    | "\($from) \($to) \($k)"
  ),
  (
    .entities | to_entries[] | .value as $e
    | ($e.relations // [])[]
    | (.target | prefix) as $to
    | select($to != $e.domain)
    | "\($e.domain) \($to) \($e.name).\(.name)"
  )
' "$work/ir.json" > "$work/edges.raw" || { echo "check-layers: jq failed on the compiled IR" >&2; exit 2; }
sort -u "$work/edges.raw" > "$work/edges.txt"

if [ "$graph" = 1 ]; then
  awk 'NR == FNR { layer[$1] = $2; next } { print layer[$1] " " $1 " -> " layer[$2] " " $2 }' \
    "$work/layers.txt" "$work/edges.txt" | sort | uniq -c | sort -k2,2n -k3,3
  echo
fi

status=0

awk '
  NR == FNR { layer[$1] = $2; next }
  {
    from = $1; to = $2
    if (!(from in layer) || !(to in layer)) { print "unknown domain: " $0; bad++; next }
    if (layer[from] == "none") { seen[from]++; if (seen[from] == 1) { print "no layer directory: " from; bad++ } next }
    if (layer[to] == "none") next
    if (layer[to] + 0 > layer[from] + 0) {
      printf "upward: %s (layer %s) -> %s (layer %s) via %s\n", from, layer[from], to, layer[to], $3
      bad++
    }
  }
  END { if (bad) { printf "%d layering violation(s)\n", bad; exit 1 } }
' "$work/layers.txt" "$work/edges.txt" || status=1

# rule 2: no cycle among the dependencies inside one layer
awk 'NR == FNR { layer[$1] = $2; next } layer[$1] == layer[$2] && layer[$1] != "none" { print $1, $2 }' \
  "$work/layers.txt" "$work/edges.txt" | sort -u > "$work/same-layer.txt"
if [ -s "$work/same-layer.txt" ]; then
  if ! tsort "$work/same-layer.txt" > /dev/null 2> "$work/tsort.err"; then
    echo "cycle inside a layer:" >&2
    cat "$work/tsort.err" >&2
    status=1
  fi
fi

if [ "$status" = 0 ]; then
  echo "layers: $(wc -l < "$work/layers.txt") domains, $(wc -l < "$work/edges.txt") dependencies, every one points to its own or a lower layer, no cycle inside a layer"
fi
exit "$status"
