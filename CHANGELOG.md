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
- Removed: the manifest of an external viewer at the repository root and the checker subcommand
  that answered that viewer's protocol. orgspec knows no viewer; a viewer keeps its own manifest
  of orgspec and reads `orgspec check --json`.
