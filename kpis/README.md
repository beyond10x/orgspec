# KPI catalogue

`catalogue.yaml` (format `orgspec-kpis/1`) lists the KPIs of every building block: what each
measures, how it is computed, its direction under each posture, whether it is a gate, and
whether a tool computes it today. `scripts/check-kpis.sh` holds the file against the blocks and
the events they declare.

```yaml
format: orgspec-kpis/1
kpis:
  - id: decision_latency_seconds  # snake_case, unique
    block: decision               # a block of blocks/system.yaml, without the `org.` prefix
    measures: quality             # correctness | completeness | quality
    over: decision_class          # what one value belongs to; see below
    unit: seconds                 # share | count | seconds | days | money | ratio | points
    summary: Time a decision waits for its decider.
    how: Seconds from DecisionOpened to DecisionTaken, per decision class.
    direction: lower              # higher | lower
    postures: { explore: ignore, scale: lower, operate: lower }   # optional overrides
    gate: false                   # true: must be 0, or 1 for a share that must be complete
    reads: [org.decision.DecisionOpened, org.decision.DecisionTaken]
    status: simulated             # checked (orgspec check counts it from a design) | simulated | planned
```

- **Three measures.** Correctness is a count that must be 0. Completeness is a share that should
  be 1. Quality is a number with a direction.
- **Three postures.** An organization that explores, one that scales and one that operates want
  different things from the same number: idle capacity is a cost when operating and room to
  learn when exploring. `postures` overrides `direction` per posture; `ignore` leaves the KPI
  out.
- **Nobody is scored.** `over` names what one value of the KPI belongs to: an organization, a
  unit, a role, a position, a decision class, a policy, an offering, a relationship, a goal, an
  asset, a budget, a process, an agreement, a forum, a channel, a risk, an obligation, a design
  or a type. It is never a party, a membership or an assignment, and the check refuses a KPI that
  says so. A position with one seat is close to a person; a report for people outside the unit
  groups such positions by role.
- **`reads`** lists the events the KPI is computed from. A KPI of status `checked` reads a design
  document and may list none.
