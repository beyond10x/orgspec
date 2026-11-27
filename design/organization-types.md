# orgspec: organization types (draft 1, 2026-11-27)

Status: design, with a first checker (`crates/orgspec`) and the public types in `types/`.

## The problem

The blocks are generic: a unit, a position, a decision right. Real organizations come in kinds.
A product company, a clinic, a cooperative and an open-source project each have their own words,
their own kinds of unit and role, parts that must exist, and rules that hold in every
organization of that kind. One particular organization adds its own on top.

Three things are wanted at once:

1. A kind of organization, and one particular organization, can say what is special about it.
2. Every generic tool (the checker, the KPIs, the simulator, the renderer) keeps working on it.
3. What is private about one organization never enters a public repository.

## The principle: a type narrows

> Every organization of a type is a valid organization of each type it extends, down to `base`.

`base` is the type with no kinds and no rules beyond the blocks. A type may add; it never
removes and never widens:

- it may name things, add kinds, add attributes, add rules, add defaults, add KPIs, add
  extension domains;
- it may not remove a block, remove or loosen a rule of a type it extends, widen a bound, or
  change what a block means.

What follows:

- **Generic tools need no knowledge of a type.** A design of any type is a design of `base`.
- **Types compose.** A rule only removes designs from the set of valid ones. So a type may
  extend several types at once, and the result is the designs valid under all of them. The
  order does not matter. Two types that cannot both hold (one requires at least two of a part,
  the other at most one) are refused when the type is checked, not when a design is.
- **No override syntax exists.** A type document has no way to write "drop rule X". Narrowing
  holds by construction, and the checker only has to compare bounds.

The planning profiles of AEP follow the same rule for a different subject: a profile that
extends another may only make completion harder.

## What varies between kinds of organizations

Nine things were found to vary. Each has one place.

| what varies | example | where it goes |
|---|---|---|
| words | a unit is called a squad, an offering a matter | `vocabulary` |
| kinds of a block | a squad is a line unit grouped by offering | `kinds` |
| extra fields | a clinic room has a hygiene class | `attributes` of a kind |
| parts that must exist | one lead position in every team | `rules` (`parts`) |
| what must be linked | every asset of kind register has one accountable position | `rules` (`links`) |
| what must never be | an agent decides no irreversible class | `rules` (`forbid`, `separate`) |
| what is assumed when unsaid | the head of a unit answers for its unowned assets | `defaults` |
| extra entities with a lifecycle | a release, a shift, a case file | `extensions` (ESS domains) |
| how it is measured | releases per week, beds occupied | `kpis` |

A tenth varies per organization and is never part of a public type: where the records of the
real organization come from. That is a binding; see "What stays private".

## A type document

Format `orgspec-type/1`. Example (`types/agent-operated.yaml`, shortened):

```yaml
format: orgspec-type/1
type: agent-operated
origin: invented
extends: [base]
summary: An organization in which agents hold positions beside humans.
vocabulary:
  position: seat
kinds:
  steward:  { block: role, summary: "The human role that answers for agent-held positions." }
  runbook:  { block: process, attributes: { reviewed_every: { type: duration, required: true } } }
rules:
  - id: agent_decides_only_reversible
    summary: A position held by agents decides no irreversible decision class.
    rule: forbid
    carries: decides
    from: { block: position, held_by: agent }
    to:   { block: decision_class, where: { reversible: false } }
  - id: agent_position_escalates
    summary: Every position held by agents has somewhere to escalate to.
    rule: links
    each: { block: position, held_by: [agent, mixed] }
    carries: escalates
    direction: out
    min: 1
  - id: agent_share_bounded
    summary: At most three in four seats of a mixed position are held by agents.
    rule: value
    each: { block: position, held_by: mixed }
    field: agent_share
    max: 0.75
defaults:
  - id: steward_answers
    summary: An agent-held position without an answers_to link answers to its unit's steward.
    each: { block: position, held_by: agent }
    carries: answers_to
    to: { block: position, role_kind: steward, same: unit }
kpis:
  - id: agent_held_seat_share
    block: position
    measures: quality
    unit: share
    summary: Share of seats held by agents.
    direction: higher
```

### `extends`

A list of type names. Every type extends `base`, directly or through another type. The chain
has no cycle. `base` is built into the checker. Every other name is resolved on the type path:
each directory of `--type-path` and of `ORGSPEC_TYPE_PATH`, `types/` under the current
directory, and `types/` beside the directory of the design. A name found in two files is
refused.

### `vocabulary`

Display words for blocks and for what a link carries. A tool shows them; no rule reads them.
The words of a type that is extended later win over those of an earlier one.

### `kinds`

A kind is a name for parts of one block.

- `block`: the block the kind belongs to. Required.
- Dimensions of the block the kind fixes, with their values: for a unit `nature` and `basis`,
  for an asset `class`, for a decision class `reversible`. A part of the kind that states
  another value is refused (`kind_dimension_mismatch`). A kind that fixes a dimension to another
  value than the kind it refines is refused (`kind_dimension_conflict`).
- `is`: another kind of the same block that this kind refines. A part of the kind counts as a
  part of every kind it refines, so a rule about the wider kind covers it.
- `attributes`: extra fields of parts of the kind. Each has a `type` (`string`, `integer`,
  `number`, `boolean`, `duration`, `enum` with `values`) and `required`. A part with an
  attribute its kind does not declare is refused (`unknown_attribute`).
- A kind name is unique across the types a type extends (`kind_declared_twice`).
- A design of a type may leave `kind` off a part. A type that wants every unit to have a kind
  says so with a rule.

### `links`

Names of links the type adds to what a link may carry, each with a `summary`:
`links: { proposes: { summary: "…" }, approves: { summary: "…" } }`. A generic tool draws them
and applies no rule of its own to them. A name the blocks or an extended type already use is
refused (`link_declared_twice`).

### `rules`

A closed vocabulary of rule forms. Every rule has an `id` (snake case, unique across the
extended types), a `summary`, and `gate` (default `true`). The id is the name of a KPI: the
count of the rule's violations. A gate must be 0 for the design to be valid. A rule with
`gate: false` is counted and reported and refuses nothing. A type may turn `gate: false` of a
type it extends into `true`, never the reverse.

A selector picks parts. Its keys: `block`; `kind` (a part of that kind or of a kind that
refines it); `role` (the name of the role part a position plays); `role_kind` (the kind of that
role part); `where`, a map of field or attribute values; and any other key, read as one more
field of the part (`held_by: agent` is `where: { held_by: agent }`). All given keys must match.
A value may be a list, meaning any of them. An empty selector matches every part.

- `parts`: the number of parts matching `match` lies in `min`..`max`. With `per`, the count is
  taken inside each part matching `per`: positions `in` that unit, units `in` that unit or
  organization.
- `links`: for each part matching `each`, the number of links that carry `carries` in
  `direction` (`in` or `out`), with the other end matching `other` if given, lies in
  `min`..`max`.
- `forbid`: no link carrying `carries` runs from a part matching `from` to a part matching `to`.
- `separate`: no part has links carrying two different entries of `carries` to the same target
  matching `to`. This is the separation of duties: who proposes does not approve.
- `within`: both ends of every link carrying `carries` lie in the same `scope`: `unit`,
  `parent` (units with the same parent) or `organization`.
- `span`: no position has more than `max` positions reporting to it.
- `depth`: no chain of `reports_to` is longer than `max`.
- `value`: for each part matching `each`, the field or attribute `field` lies in `min`..`max`
  or is one of `one_of`.

A rule form that cannot be written with these is a reason to add a form here, in public, in
neutral words. It is never a reason for a private escape hatch.

### `defaults`

A default adds a link the design did not write. For each part matching `each` with no link
carrying `carries` (out), it adds one to the part matching `to`. `same: unit` restricts `to` to
the unit of the part, `same: parent` to units with the same parent. When no part or more than
one part matches `to`, nothing is added and the rule that needs the link reports as usual.

Defaults are applied first; the completed design is then checked against every rule, the
structural rules included. So a default can satisfy a structural rule: the `answers_to` link a
default adds counts for `agent_without_accountable_human`. A default writes down what the
type's organizations leave unsaid. Each default is also a KPI, the count of links it added.
Default ids share one pool with rule ids.

### `kpis`

KPIs of the type: `id`, `summary`, `block`, optionally `of_kind` (a kind of that block),
`measures` (correctness, completeness or quality), `unit`, `direction` and optionally `target`.
A target is not a rule. A KPI over a party, a membership, an assignment or a competence is
refused (`kpi_over_party`).

### `extensions`

Entities with their own lifecycle and commands do not fit in a type document. They are ESS
domains:

```yaml
extensions:
  - domain: clinic.shift
    extends: assignment        # the block it sits beside
    path: ess/3-people/shift.yaml
```

- An extension lives in an ESS system of its own, beside the type document. That system
  carries a copy of `blocks/` at one release, proved unchanged by the digest of the release
  (`scripts/blocks-digest.sh`).
- An extension domain sits in the layer directory of the block it extends. It may depend on
  that layer and lower ones. No block depends on it: the blocks are a byte-equal copy.
  `scripts/check-layers.sh --path <system>` holds both.
- An extension entity is keyed by the identifier of the generic entity it extends, or names it
  as a field. A release is keyed by a work item; a ward is keyed by a unit. So a generic tool
  sees a work item and a unit, and a tool of the type sees more.
- An extension never restates a field of the block it extends.

### Levels of use, cheapest first

0. No type: a design of `base`.
1. A public type, or several.
2. A type document of one's own that extends public ones. No ESS is written.
3. Extension domains.
4. Bindings to the organization's records, for an observed design and observed runs.

Most organizations stop at 2.

## Checking

- `orgspec type check <type>` resolves the extended types and refuses: an unknown type, a
  cycle, a kind declared twice, a kind that fixes a dimension its block does not have, a rule
  id declared twice, a rule form outside the vocabulary, a `gate: true` turned to `false`, and
  bounds that cannot hold together. A document of the wrong shape (a missing required key, an
  id that is not snake case, a key written twice) ends with exit 2 and the first error.
- Not checked yet: the block of a link's ends beyond `accountable`, `decides` and `demand`; a
  cycle of units through `in`; the keys of `vocabulary`.
- `orgspec check <design>` resolves the design's type, applies the defaults, checks the
  structural rules of `design/building-blocks.md` and every rule of every extended type, and
  prints one line per violation with the rule id and the part. Exit 1 when a gate is not 0.
  `--json` prints the counts as KPIs.
- In ESS: `org.type` holds a type as rows (kinds, rules, defaults, revisions), and `org.design`
  names the type revision a design was committed under. Lowering a type document to commands of
  `org.type` follows the lowering of a design.

## What stays private

The line is between a kind of organization and one organization.

| | holds | where |
|---|---|---|
| public | the blocks, types of kinds of organizations, invented examples, the tools | this repository |
| internal | one organization's type, its design (units, positions, roles, rules), its extension domains | a private repository of that organization |
| restricted | who holds which position, proficiencies, observed runs, bindings to record systems | the same private repository, or a narrower one |

- **A design names no party.** `held_by` says human, agent or mixed. Who holds a seat is an
  assignment, a separate document (`orgspec-assignments/1`) in the restricted tier. So a design
  can be read by everyone inside the organization, and the assignments by fewer.
- **References point one way.** A private type extends public types by name. A public file
  never names a private type, a private path, a host or a person.
- **The tools read private files; they never write them into this repository.** The type path
  and the design path are arguments. No command copies, anonymizes or exports a private
  document into the public tree.
- **Structure is private too.** A copy of a real design with the names replaced still shows
  the number of seats, the spans, who decides what. So a public example is written from a kind
  of organization and carries `origin: invented`. It is never derived from an instance.
- **A KPI is read over a position, a role or a unit, never over a party.** Nobody is scored.
  The blocks declare no KPI that reads a party, and a type that adds one is refused.
- **An unmapped record is kept, not forced.** When a binding meets a record no block or kind
  takes, it writes the record to an unmapped list with the reason. No enum is stretched. The
  unmapped list is the input for the next kind, or for a change of the blocks.

### Guards

1. `scripts/check-public.sh`, in the gate of this repository:
   - every file under `types/` and `organizations/` states `origin: invented`;
   - no file contains a home directory path, an e-mail address outside `example.org`, or an IP
     address;
   - no file contains a term of the deny list, when one is configured.
2. The deny list is a file outside this repository, named by `ORGSPEC_DENYLIST`: one term per
   line, matched without regard to case; a line starting with `=` matches the exact word only.
   An organization builds it from its own registers (its name, its products, its customers, its
   people, its hosts). The list is as private as the things it names: it is never committed
   here, and no digest of it is published, because a digest of a short name can be guessed.
3. Without a deny list the script says so in its last line. A contributor who has read a
   private source sets one. A hook (`scripts/install-hooks.sh`) runs the script before every
   commit and every push.
4. A gap found in private work reaches this repository as a gap report in neutral words: the
   block, the missing field, rule form or kind, and the smallest addition. It passes the same
   script before it leaves.

### For agent sessions

- A session that works in this repository does not read a private repository unless its
  operator names it.
- A session that has read a private source writes public text from first principles: what any
  organization of that kind has. It copies no name, no number and no structure.
- It sets `ORGSPEC_DENYLIST` and runs `scripts/check-public.sh` before it reports.
- Every sub-agent brief names these three rules.

## How one organization adopts it

1. Pick the public types that describe its kind.
2. In a private repository, write `types/<name>.yaml` that extends them: its words, its kinds,
   its rules, its defaults.
3. Write its design: `organizations/<name>.yaml`, structure only.
4. Run `orgspec check --type-path types organizations/<name>.yaml`. Each violation is either a
   defect of the organization or a rule it does not hold; decide which.
5. Later: assignments, extension domains, bindings, observed runs, simulation and search.

## Open questions

- Q1 A lock file that pins each extended type by digest, so a change of a public type shows as
  a diff in the private repository. Default taken: not in the first checker; names only.
- Q2 Whether a kind may fix an attribute of a kind it refines. Default taken: yes, as a `value`
  rule.
- Q3 Whether `defaults` may add parts and not only links. Default taken: links only.
- Q4 Rules over a run (a decision taken without the right) beside rules over a design. Default
  taken: run rules are KPIs of the blocks; a type adds run KPIs under `kpis`, not `rules`.

## Gaps found by a first private type

A first organization wrote a private type and design against `orgspec-type/1` and reported these
gaps. None is built. Each is stated without the organization.

- G1 `defaults`: a default adds a link out of the selected part only. Wanted: `direction: in`.
- G2 Selectors: no key selects a part by the kind of the unit it sits in. Wanted: `in_kind`.
- G3 `border`: no selector or rule form reads it. Wanted: a selector key `border: true|false`.
- G4 `process`: no rule form reads `steps`. Wanted: a form that requires one step before
  another.
- G5 Holders: no rule form reads who holds a position, and `separate` compares parts. Wanted:
  an assignments document and a rule form `holders` (two positions not held by one party).
- G6 `forum`: `convenes` takes one unit, and a forum that settles no class is refused as
  unreachable. A meeting of several units that only informs cannot be written.
- G7 `value` reads the fields of a part, not of a link (`after`, `limit`, `mode`).
- G8 `kpis`: `direction` has no value for a measure that is read only beside another one.
