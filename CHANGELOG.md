# Changelog

## Unreleased

- Building blocks of an organization as one ESS system (`blocks/`, system `org`): 30 domains in
  13 layers, with a layer check and its negative fixtures.
- Design documents: `design/building-blocks.md`, `design/organization-types.md`.
- Formats `orgspec-organization/1` (a design) and `orgspec-type/1` (an organization type).
- The checker `orgspec check` and `orgspec type check` (`crates/orgspec`).
- Public types: `agent-operated`, `product-company`, `regulated`, `professional-services`,
  `cooperative`, `open-source-project`.
- Eight invented example designs and one refused design per rule (`organizations/`).
- KPI catalogue (`kpis/catalogue.yaml`, format `orgspec-kpis/1`).
- `scripts/check-public.sh`: refuses home paths, e-mail addresses, IP addresses, types and
  designs without `origin: invented`, and terms of a deny list kept outside the repository.
- Conformance of the blocks on the ESS interpreter (`scripts/check-blocks-conform.sh`), with 28
  hand-written scenarios and a list of the synthesized scenarios known to fail
  (`blocks/known-failing.txt`).
- The test step runs under cargo-nextest: a test is killed at 60 s, the step fails over 120 s.
- CI: `.github/workflows/gate.yml` runs `task check`; a pull request that changes only planning
  or prose files passes without it (`scripts/planning-only.sh`).
  `.github/workflows/shared-gates.yml` runs the organization's shared source gates.
- `specapp.yaml`: the manifest (`specapp-pack/1`) that specapp reads this repository with: the
  blocks, the KPI catalogue, and two document classes, `organization` (`organizations/*.yaml`,
  drawn as parts and links, a part sits `in` another) and `type` (`types/*.yaml`, records). It
  names no presentation, so a block takes the colour of its layer.
- `orgspec specapp describe` and `orgspec specapp check <design>`: the checker answers specapp's
  tool protocol. `check` prints `specapp-findings/1`: `valid`, one finding per violation with
  its rule, gate, subject (`part:<name>`, `link:<from>-><to>:<carries>`, or `document` with the
  name kept at the start of the message) and message, and the count of every rule. Exit 0 with
  the document, exit 2 with `{error}` when the design cannot be read or the verb is unknown.
- `specapp.yaml` names the tool: `target/debug/orgspec specapp`, verb `check`. specapp builds
  nothing: run `cargo build -p orgspec` first; `specapp pack check` says when the binary is
  missing.
