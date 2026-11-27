# orgspec: rules for agent sessions

orgspec specifies an organization from first principles: building blocks in ESS, organization
types that narrow them, example designs, and a checker. Read `design/building-blocks.md` and
`design/organization-types.md` before changing anything.

This repository is public. That decides most of the rules below.

## Nothing here describes a real organization

- Every file under `types/` and `organizations/` carries `origin: invented`.
- An example is written from a kind of organization (a bakery, a clinic, a studio). It is never
  derived from a real one, and never a real one with the names replaced: the number of seats,
  the spans and who decides what identify an organization as well as its name does.
- A session that works here does not read a private repository unless its operator names it.
- A session that has read a private source writes public text from first principles: what any
  organization of that kind has. It copies no name, no number and no structure.
- Such a session sets `ORGSPEC_DENYLIST` to the deny list of that organization and runs
  `scripts/check-public.sh` before it reports. The deny list is a file outside this repository.
  It is never committed here and no digest of it is published.
- A gap found in private work is reported here in neutral words: the block, the missing field,
  rule form or kind, and the smallest addition.
- No command of this repository copies, anonymizes or exports a private document into it.
- Every sub-agent brief names these rules.

## The blocks (`blocks/`)

- One ESS system, `org`. One domain per block, one file per domain, one directory per layer.
- Dependencies point down only. `scripts/check-layers.sh` holds it; its negative fixtures must
  fail. When a lower block would name a higher one, turn the edge around: the higher block holds
  the identifier of the lower one.
- Identifiers of every entity live in `blocks/0-foundation/shared.yaml`. An identifier counts as
  a dependency on the domain whose entity it identifies.
- No `owner` field on any entity. Who answers for a thing is a row of `org.responsibility`.
- A seat is not a person. Reporting lines, decision rights and responsibilities name a
  `PositionId`. Only `membership`, `assignment` and `competence` name a party or a membership.
- The field name `kind` is reserved for `Optional<org.shared.KindName>`, the kind an organization
  type gives. A block stores it and never branches on it. A block's own enumeration has another
  field name (`nature`, `basis`, `class`, `tie`, `form`, `category`, `activity`, `duty`,
  `measures`). `party.kind` is the one exception. The `tag: kind` of a union is a wire tag and
  is not covered.
- Default deny: a missing right, link or rule refuses with a named error.
- Where ESS cannot express a rule, the file says so in a comment that starts `# UNMAPPED:`,
  with the error code of ESS and what was taken instead. `design/building-blocks.md` § "What ESS
  lacks here" lists them by cause.
- A KPI is read over a position, a role or a unit, never over a party.

## Types and designs

- A type narrows: every organization of a type is valid under each type it extends. There is no
  override syntax, and none is added.
- A rule a type needs and the rule forms cannot say is a reason to add a rule form, in public,
  in neutral words.
- Every rule of the checker and of a public type has a broken design under
  `organizations/broken/<rule_id>.yaml` that is refused for that rule alone.

## Gate

- `task check` is the gate. Parts: `check:public`, `check:blocks`, `check:scripts`,
  `check:rust`, `test`, `check:types`.
- The test step ends within 120 s and no test runs over 60 s. An expensive product is made once
  per digest of its sources (`scripts/check-blocks-conform.sh` keeps the synthesized suite under
  `target/blocks-conform/`), never once per test.
- The tests make no network request and no model call.
- No Python in anything that stays here. Scripts are POSIX sh; tools are Rust.
- Manifests and configuration files carry no comments.

## Delivery

- The remote is `github.com/beyond10x/orgspec`. Only the organization's bot writes a ref, and
  every commit has the bot as author and committer. Commit with
  `b10x-gates bot -- commit -F <file>`, push with `b10x-gates bot -- push`, open a pull request
  with `b10x-gates gh -- pr create`.
- `.github/workflows/shared-gates.yml` runs the organization's source gates, pinned by commit.
- `.github/workflows/gate.yml` runs `task check` with the toolchain `rust-toolchain.toml` names,
  `ess` 0.99.2 and cargo-nextest 0.9.143. It has no deny list, so `check:public` runs its other
  four checks there.
- A pull request that changes only planning or prose files passes `gate.yml` without
  `task check`. The path list is `scripts/planning-only.sh`; `scripts/planning-only.test.sh`
  holds both sides of it.
- `main` requires the checks `common / Security and privacy` and `gate`.

## Layout

- `blocks/`: the ESS system; `blocks/scenarios/`: hand-written scenarios.
- `types/`: public organization types (`orgspec-type/1`).
- `organizations/`: invented example designs (`orgspec-organization/1`);
  `organizations/broken/`: one refused design per rule.
- `kpis/`: the KPI catalogue (`orgspec-kpis/1`).
- `crates/orgspec/`: the checker.
- `scripts/`: the checks of the gate and their fixtures.
- `.github/workflows/`: the CI gate and the caller of the shared source gates.
- `design/`: the two design documents.
