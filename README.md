# orgspec

An organization, specified from first principles.

orgspec answers six questions about any organization: who belongs to it, how it is divided, who
may decide what, what it gives to whom, how work moves through it, and who answers for what. Each
answer is a building block with a contract in [ESS](https://github.com/beyond10x/ess). An
organization is a composition of blocks, written as one document and checked by one command.

It is the sibling of [agenticspec](https://github.com/beyond10x/agenticspec), which does the same
for agentic software development.

## What is in it

- **Blocks** (`blocks/`): 30 ESS domains in 13 layers, from `party` and `organization` up to
  `responsibility`, `design` and `kpi`. Dependencies point down only.
- **Organization types** (`types/`): what holds for every organization of one kind. A type adds
  words, kinds of units and roles, rules and defaults. It never removes or loosens anything, so
  types combine: `[agent-operated, product-company]` means both hold.
- **Example designs** (`organizations/`): a studio, a cooperative bakery, a clinic, an
  open-source project, a product company, a shop run mostly by agents, a design agency and a
  town office. All invented.
- **A checker** (`crates/orgspec`): `orgspec check` and `orgspec type check`.
- **A KPI catalogue** (`kpis/`): what to measure per block. No KPI is read over a person.

## Try it

Requires Rust (the toolchain in `rust-toolchain.toml`).

```sh
cargo build -p orgspec
target/debug/orgspec check organizations/studio.yaml
target/debug/orgspec check organizations/broken/agent_without_accountable_human.yaml
target/debug/orgspec type check types/agent-operated.yaml
```

The second command exits 1 and names the rule and the part. Each file under
`organizations/broken/` is a design that is valid except for the one rule in its name.

## Three statements the blocks hold

1. **A seat is not a person.** Rights, responsibilities and reporting lines attach to a
   position. Who holds the position is a separate fact, so a position exists while it is vacant
   and a person can leave without a duty getting lost.
2. **Accountability is recorded in one place.** No block has an owner field. One block,
   `responsibility`, says which position is accountable for which thing, and there is one such
   position per thing.
3. **Every chain ends at a human.** An agent may hold a position. Its position answers to
   another one, and that chain reaches a position a human holds.

## Describing your own organization

1. Pick the public types that describe its kind.
2. In a repository of your own, write a type that extends them: your words, your kinds, your
   rules.
3. Write the design: units, roles, positions, who decides what. No names of people.
4. `orgspec check --type-path <this repository>/types --type-path types organizations/<name>.yaml`

Your type, your design and who holds which position stay in your repository.
[design/organization-types.md](design/organization-types.md) § "What stays private" says where
the line runs and which guards hold it.

## Documents

- [design/building-blocks.md](design/building-blocks.md): the blocks, the design document, the
  structural rules, simulation, KPIs.
- [design/organization-types.md](design/organization-types.md): types, rule forms, composition,
  privacy.
- [kpis/README.md](kpis/README.md): the KPI catalogue.
- [AGENTS.md](AGENTS.md): rules for agent sessions that work here.

## Status

A first draft. The blocks validate and run on the ESS interpreter; 25 synthesized scenarios are
known failing and named in `blocks/known-failing.txt` with their cause. The checker holds the
structural rules and the rules of the public types. Not built yet: lowering a design to commands
of the `design` block, the simulator and its search, a rendered view, a documentation site, and
lifting an observed organization into a design.

## Gate

```sh
task check
```

Needs [Task](https://taskfile.dev), the `ess` CLI at the version `blocks/ess-inputs.yaml` names,
`jq`, `yq` (mikefarah, v4) and cargo-nextest. `.github/workflows/gate.yml` runs the same command
on every pull request and on `main`.

## License

Apache-2.0. See [LICENSE](LICENSE).
