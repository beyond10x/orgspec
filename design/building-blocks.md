# orgspec: building blocks of an organization (draft 1, 2026-11-27)

Status: first wave. Built: the blocks in `blocks/`, the checker in `crates/orgspec/`, six public
types, eight example designs and the KPI catalogue. Not built: steps 4 to 8 of the build order at
the end.

## What it is

- A specification, written in ESS, of what an organization consists of.
- It declares building blocks: party, organization, unit, role, position, decision right, goal,
  offering, process, forum and the rest of the list below.
- Each block is a contract: the commands it accepts, the events it emits, the views it serves.
- An organization design is a document that composes blocks into one concrete organization.
- A design is checked against the specification before anybody is moved.
- A design is simulated, and the run is scored with KPIs.
- A search changes the design and keeps the variant with the better score.
- A system that implements a block (a directory, a planning tool, an approval service) proves it
  against the block's contract.
- An organization type narrows the blocks for one kind of organization. It is described in
  `design/organization-types.md`.

It is the sibling of agenticspec, which does the same for agentic development. agenticspec says
how agents, models, tools and channels are composed into an architecture. orgspec says who
exists, how the whole is divided, who may decide, who answers for what and how work crosses the
divisions. The two meet at the party: see "Where it meets agenticspec".

## First principles

An organization exists when more than one party pursues something together for longer than one
task. Then six questions have to be answered, by a rule or by habit. Each block is the answer to
a part of one question.

1. **Who is it, and who is in it?** A boundary and members that change while it persists:
   `party`, `organization`, `membership`.
2. **How is the whole divided?** Nobody does everything, so the whole is cut into groups and
   seats: `unit`, `role`, `position`, `assignment`, `competence`.
3. **Who may decide what?** Division creates the question who settles what: `decision`,
   `delegation`, `policy`.
4. **What is it for, and for whom?** What it gives to parties outside it, and what it wants to
   be true: `offering`, `relationship`, `goal`.
5. **How does work cross the divisions?** Every cut made in question 2 is a seam that work must
   cross: `process`, `work`, `agreement`, `forum`, `channel`, `escalation`.
6. **What does it spend, and how does it know it works?** `asset`, `budget`, `responsibility`,
   `risk`, `obligation`, and the KPIs.

Three statements follow from the questions and hold in every block:

- **A seat is not a person.** A position exists while it is vacant; an assignment puts a member
  in it. Responsibility, decision rights and reporting lines attach to positions, so they
  survive a change of holder.
- **Accountability is recorded in one place.** No block carries an `owner` field. Who answers
  for an offering, an asset, a goal, a process, a policy, a budget or an agreement is a row of
  `responsibility`. So "who owns what" is one view and "what has no owner" is one KPI.
- **Every chain ends at a human.** A position may be held by an agent. The assignment of an
  agent names the position that answers for it, and that chain ends at a position held by a
  human. agenticspec states the same for a delegation.

## Three levels, two sources

```text
blocks ──────▶ design ──────▶ run ──────▶ KPIs
(spec, ESS)    (one document)  (events)    (numbers, gates)

   organization type ──▶ narrows the blocks for one kind of organization

                 simulated run ──┐
                                 ├──▶ one event vocabulary ──▶ the same KPIs
                 observed run  ──┘
```

- **Blocks**: types. What a position is, what a decision right is, what may name what.
- **Design**: an instance. Which units, which positions, who decides what, how often a forum
  meets.
- **Run**: what happened over time, as commands and events: requests, hand-offs, decisions.
- A **simulated run** comes from the specification, a design and a behaviour profile.
- An **observed run** comes from a real organization's records, lifted into the same events.
- Both produce the same events, so one KPI definition scores both.
- An **observed design** is a design document lifted from a real organization's records. Every
  lifted field carries its attribution: known, inferred or not recorded.

## The blocks

One domain per block, one directory per layer. Dependencies point down only:
`scripts/check-layers.sh` reads `ess specify compile` output and refuses a dependency on a higher
layer, a cycle between domains of one layer, and a domain outside a layer directory. A dependency
is a type of another domain, the identifier of another domain's entity (an identifier in `shared`
counts for the domain whose entity it identifies), or a relation. Its negative fixtures must fail.

When a lower block would name a higher one, the edge is turned around: the higher block holds
the identifier of the lower one. A unit does not list its positions; a position names its unit.
An offering does not name its accountable position; a responsibility names both.

The field name `kind` is reserved. On a unit, a role, an offering, an asset, a process, a work
item and a forum it holds the name an organization type gives (`org.shared.KindName`); the block
stores it and never branches on it. A block's own enumeration has another name: a unit's
`nature` and `basis`, an asset's `class`, a relationship's `tie`, a channel's `form`, a policy's
`category`, a step's `activity`. The one exception is `party.kind`, which mirrors
`agentic.principal`; a party lies outside every organization, so no type names its kind.

Each line: the block and what it is from first principles.

### 0 foundation

- `shared`: identifiers of every entity; the kind of a party; attribution (known, inferred, not
  recorded); audience; level; money; a share; a period; the kind name an organization type gives.

### 1 party

- `party`: who can act and answer for it. A human, an agent, an organization, a service. A party
  exists outside any organization; the same party can be a member of two.
- `organization`: the bounded whole. A name, a purpose, a form, an optional parent organization.
  It is itself a party, so it can be a customer, a supplier or a member of another organization.

### 2 structure

- `unit`: a grouping inside an organization. It has a nature (line, governing, community,
  temporary) and a basis of grouping (function, offering, market, region, process, project). A
  board is a governing unit. Units form a tree.
- `role`: a named bundle of expectations: a purpose and what its holder is expected to do. A
  role belongs to the organization, not to a unit, so two units can use it.
- `position`: a seat in a unit that plays a role, with a number of seats and the position it
  reports to. It exists while vacant.

### 3 people

- `membership`: a party belongs to an organization on a basis (employed, contracted, volunteer,
  owner, operated agent, seconded). Invited, active, suspended, ended.
- `assignment`: a member holds a position, with a share of its time and a period. An agent's
  assignment names the position that answers for it.
- `competence`: a skill, the level a party has it at, and the level a role requires.

### 4 authority

- `decision`: a class of decisions (spend, hire, price, release), the right to take it (who,
  in which unit's scope, in which mode: sole, advice, consent, majority, unanimous), and the
  record of a decision taken: the options, the one chosen, who decided, what it supersedes.
- `delegation`: a position passes a decision class to another position, with a limit and an
  end. A delegation never exceeds what the delegating position holds.
- `policy`: a statement that binds a scope: a rule, a principle or a standard. Adopted by a
  decision, in force until retired. A waiver exempts one unit or position until a date.

### 5 value

- `offering`: what the organization gives to a party outside it, or to its own units: a
  product, a service, a programme.
- `relationship`: a party outside the organization and the kind of tie: customer, supplier,
  partner, regulator, funder, beneficiary. An engagement says which offering the party takes.

### 6 direction

- `goal`: what the organization wants to be true by a date. A goal tree with key results: a
  measure, a baseline, a target, a current value.

### 7 resources

- `asset`: what the organization has and must keep: physical, digital, information, financial,
  intangible. With a criticality.
- `budget`: an amount given to a unit for a period, optionally for a goal, approved by a
  decision, with its spend.

### 8 work

- `process`: the way a kind of work is done: steps in order, each performed by a role. A step
  is doing, deciding, reviewing or waiting.
- `work`: demand and its fulfilment. A request from a relationship or a unit, a work item at a
  position, a hand-off between positions, a commitment with a due date that is kept, broken or
  renegotiated.
- `agreement`: what one unit provides to another unit or to a relationship, with terms: a
  measure and a target. An agreement is the contract at a seam.

### 9 coordination

- `forum`: a recurring meeting of a unit or of positions, with a purpose, a cadence and the
  decision classes it may settle. A meeting is one occurrence.
- `channel`: a path for messages between positions and units, with an audience (restricted,
  internal, external).
- `escalation`: what happens when a work item or a decision is stuck: a path of positions, each
  with a time limit, and the record of one escalation moving along it.

### 10 control

- `responsibility`: who answers for what. A position is accountable for, responsible for,
  consulted on or informed about a subject. The subject is an offering, an asset, a goal, a
  process, a policy, a budget, an agreement, a relationship, a unit, a risk or an obligation.
  Accountability is an entity of its own, keyed by the subject, so a second accountable
  position for one subject cannot be recorded; it is held or vacant, and a vacant row is the
  record that nobody answers. The other three duties are rows beside it.
- `risk`: what could go wrong with a subject, how likely, how bad, and the treatment.
- `obligation`: a duty from outside (a law, a contract, a standard) and the control that meets
  it, with its evidence.

### 11 composition

- `type`: an organization type as data: the kinds it names, the parts it requires and the rules
  it adds. See `design/organization-types.md`.
- `design`: the composed organization. Parts (instances of blocks), links (reports to,
  accountable for, decides, hands off to, escalates to), a border (the positions a party outside
  reaches), revisions. A revision is put under the type revisions it is checked against, so two
  revisions of one design can sit under two revisions of a type.

`org.design` holds the rules of one link or one part where the link or part is added: a link to
an unknown part, a second accountable, a second decider, an accountable that is no position, a
decider that is not allowed, demand without an offering, a part that reports to itself, a border
entry without a part. Commit refuses what is missing: an empty design, a unit or position placed
nowhere, a subject without an accountable position, a class without a decider, an agent position
that answers to nobody, a delegation without the right. `orgspec check` reads the same rules
over a whole document at once and reports all of them; the rules over a chain (cycles, the cover
of a unit, the limit of a delegation, reachability) are held by `orgspec check` alone.

### 12 evaluation

- `kpi`: a KPI as data. Name, unit, the events it reads, the block it belongs to, and its
  direction per posture (explore, scale, operate).
- `experiment`: a design, a load, a behaviour profile, faults, a search, and the result.

### Planned blocks

Each fits a layer without an upward dependency. They follow the first wave.

- `initiative` (6): a bounded effort that moves a goal, with a budget and an end.
- `capability` (7): what the organization can do, independent of who does it, with a maturity.
- `capacity` (7): the time members have per period and what it is allocated to.
- `reward` (4): what the organization pays or recognises, and for what.
- `review` (10): an audit or retrospective, its findings and the corrective actions.
- `change` (11): a reorganization as the moves between two design revisions.

## A block is a contract

- Commands, events, views and errors in ESS (`ess/37`).
- Guards carry the rules. Examples:
  - a second accountable position for one subject is refused;
  - a delegation with a limit above the delegating position's is refused;
  - a decision by a position without the right is refused;
  - an agent's assignment without a position that answers for it is refused;
  - a reporting line that closes a cycle is refused.
- Refusals are named errors, never a silent fallback.
- Default deny: no right, no decision; no link, no reach.

## How another repository uses a block

ESS (0.99.2) has no import between specifications. A consumer copies a block's types into its
own domain and proves the copy by digest (`ess-composition`). Its component declares the role it
plays: a directory service plays `party` and `membership`, a planning tool plays `goal` and
`work`, an approval service plays `decision`. The block's conformance suite then runs against
the consumer's implementation.

## Where it meets agenticspec

- `org.party` is `agentic.principal`: the same identity. A party of kind agent is an agent of an
  agentic architecture.
- `org.delegation` is the organizational source of an `agentic.delegation`: an agent acts for a
  principal because a position passed it a decision class.
- A position held by an agent is realised by a part of an agentic architecture. The design
  names the architecture; it does not describe it.
- `org.decision` is where an `agentic.question` ends when it needs a human: the decision right
  says which position answers.
- `org.work` hands a work item to a position; when the holder is an agent, the item becomes an
  `agentic.work` task.
- KPIs join at the party: the wait on a decider in orgspec is the owner wait in agenticspec.

Neither specification copies the other. The mapping is by identifier and is proved with
`ess-composition`, as for any consumer.

## Where it meets other Beyond10x specifications

Several public repositories already declare a part of an organization for their own purpose.
orgspec does not replace them. Each can play the block named here, and its implementation can
be held to the block's conformance suite.

- `mandate` declares an organization, a membership, a flat team, a delegation that is not
  passed on, an approval and a grant: it can play `organization`, `membership` and `delegation`.
- `runbook` declares a decision request with options, a recommendation, a default on silence,
  a holder and an escalation: it can play `decision`.
- `conductor` and `control-plane` declare a goal and an assignment of work: `goal`, `work`.
- `agency` declares a team, a workforce and a work item: `unit`, `position`, `work`.
- `inbox` declares a message between sessions: `channel`.
- `weft` declares a tenant, an account, a role binding and a budget: `organization`, `budget`.
- `canon` declares a protocol between parties: `agreement`.
- The planning profiles of `aep` extend one another under the rule that an extension only makes
  completion harder. Organization types follow the same rule.

What none of them declares, and orgspec adds: a tree of units, a position apart from its
holder, accountability as a record, a decision right as an entity, key results, a forum with a
cadence, a relationship to a customer or a supplier, a skill of a party, and an organization
type.

## An organization design document

Format `orgspec-organization/1`. Parts and links, the form agenticspec uses for an architecture,
so one renderer and one search serve both. Example: a product studio
(`organizations/studio.yaml`, shortened).

```yaml
format: orgspec-organization/1
organization: studio
type: base
parts:
  studio:   { block: organization, form: company }
  founders: { block: unit, nature: governing, in: studio }
  build:    { block: unit, nature: line, basis: offering, in: studio }
  lead_role:  { block: role }
  maker_role: { block: role }
  lead:     { block: position, in: build, role: lead_role, held_by: human }
  maker:    { block: position, in: build, role: maker_role,
              seats: { value: 4, vary: { min: 2, max: 8 } },
              held_by: mixed, agent_share: { value: 0.5, vary: [0, 0.25, 0.5, 0.75] } }
  app:      { block: offering }
  users:    { block: relationship, tie: customer }
  spend:    { block: decision_class }
  priority: { block: decision_class }
  weekly:   { block: forum, convenes: build, every: { value: 7d, vary: [1d, 7d, 14d] } }
links:
  - { from: maker,   to: lead,     carries: reports_to }
  - { from: maker,   to: lead,     carries: answers_to }
  - { from: lead,    to: founders, carries: reports_to }
  - { from: lead,    to: app,      carries: accountable }
  - { from: maker,   to: app,      carries: responsible }
  - { from: users,   to: app,      carries: demand }
  - { from: lead,    to: priority, carries: decides, mode: sole }
  - { from: founders, to: spend,   carries: decides, mode: consent }
  - { from: founders, to: lead,    carries: delegates, class: spend, limit: 5000 }
  - { from: maker,   to: lead,     carries: escalates, after: 2d }
  - { from: weekly,  to: priority, carries: settles }
border: [lead]
```

- Every part names its `block`: `organization`, `unit`, `role`, `position`, `decision_class`,
  `policy`, `offering`, `relationship`, `goal`, `asset`, `budget`, `process`, `agreement`,
  `forum`, `channel`, `risk` or `obligation`. A part's other fields are the block's own
  dimensions, and the attributes its kind declares.
- `type` is one type name or a list. `origin: invented` marks a document that describes no real
  organization.
- `kind` on a part is a kind of the design's type. Type `base` declares none.
- `in` places a unit in an organization or a unit, and a position in a unit. `role` names the
  role part a position plays. `convenes` names the unit a forum convenes.
- `held_by` is `human` (the default), `agent` or `mixed`; `agent_share` is the share of seats
  agents hold. A design names no party: who holds a seat is an assignment, a separate document.
- What a link carries: `reports_to`, `answers_to`, `accountable`, `responsible`, `consulted`,
  `informed`, `decides`, `delegates`, `settles`, `demand`, `serves`, `hands_off`, `escalates`,
  `funds`, and the links the design's type adds.
- Other fields of a link: `scope` (the unit a decision right is limited to), `class` (the
  decision class a delegation passes), `limit` (an amount), `mode` (`sole`, `advice`, `consent`,
  `majority`, `unanimous`), `after` (a duration) and `via` (the channel part it runs through).
- `reports_to` ends at a position or at a unit: a lead reports to a board.
- `border` lists the parts a party outside the organization reaches: a position, an external
  channel, an offering.
- A value that a search may change is written `{ value, vary }`. `vary` is a list or
  `{ min, max, step }`.
- A duration is a string with a unit: `90s`, `15m`, `24h`, `7d`.

Fields of a part, per block. Every part also takes `kind`, `summary` and `title`. A name in
brackets is a reference to another part of that block.

- `organization`: `form` (company, public_body, association, cooperative, partnership, project,
  informal), `purpose`, `in` [organization]
- `unit`: `in` [organization, unit], `nature` (line, governing, community, temporary), `basis`
  (function, offering, market, region, process, project), `purpose`
- `role`: `purpose`, `expects`
- `position`: `in` [unit], `role` [role], `seats`, `held_by`, `agent_share`, `answer_delay`
- `decision_class`: `reversible`, `impact` (low, medium, high, critical)
- `policy`: `category` (rule, principle, standard), `scope` [organization, unit]
- `offering`: `audience` (internal, external)
- `relationship`: `tie` (customer, supplier, partner, regulator, funder, beneficiary)
- `goal`: `by`, `parent` [goal], `measure`
- `asset`: `class` (physical, digital, information, financial, intangible), `criticality`
- `budget`: `amount`, `currency`, `period`, `for` [unit]
- `process`: `steps`, each with `name`, `role` [role] and `activity` (do, decide, review, wait)
- `agreement`: `provider` [unit], `consumer` [unit, relationship], `measure`, `target`
- `forum`: `convenes` [unit], `every`
- `channel`: `form` (direct, broadcast, queue, record), `audience` (restricted, internal,
  external)
- `risk`: `about` [any], `likelihood`, `impact`, `treatment` (accept, reduce, transfer, avoid)
- `obligation`: `source` (law, contract, standard), `about` [any]

A field outside this list is an attribute; a part may carry it only when its kind declares it.

Checking it:

- The document is lowered to commands of the `design` domain in a fixed order: declare the
  parts, declare the links, commit the revision. The ESS interpreter runs them. The design is
  valid when every command is accepted.
- Until that lowering exists, `orgspec check <design>` applies the structural rules directly.
  It exits 1 and prints one line per refusal. Each rule is also a gate KPI, the count of its
  refusals.

Structural rules of every design, whatever its type:

- `unknown_block`: a part names a block outside the list above.
- `unknown_carries`: a link carries something neither the list above nor the type declares.
- `unknown_link_parts`: a link's `from` or `to` names no part.
- `part_outside_organization`: a unit's `in` names no organization or unit part; a position's
  `in` names no unit part.
- `unknown_role`: a position's `role` names no role part.
- `reporting_cycle`: the `reports_to` links form a cycle.
- `subject_without_accountable`: an offering, asset, goal, process, policy, budget or agreement
  part has no incoming `accountable` link.
- `subject_with_two_accountables`: such a part has more than one.
- `accountable_not_position`: an `accountable` link starts at a part that is not a position.
- `agent_without_accountable_human`: a position held by agents, alone or mixed, has no
  `answers_to` chain that ends at a position held by humans.
- `class_without_decider`: a decision class part has no incoming `decides` link.
- `class_with_two_deciders`: two `decides` links to one class have the same `scope` (a unit
  part; none means the whole organization).
- `decider_not_allowed`: a `decides` link starts at a part that is neither a position nor a
  governing unit.
- `delegation_without_right`: a `delegates` link names a `class` that its `from` part neither
  decides nor holds by a delegation.
- `delegation_exceeds_limit`: a `delegates` link has a `limit` above the limit its `from` part
  holds for the class.
- `demand_without_offering`: a `demand` link does not run from a relationship to an offering.
- `escalation_dead_end`: a chain of `escalates` links closes a cycle, or ends at a part that
  decides no class.
- `border_without_part`: `border` names no part.
- `unreachable_parts`: no link, no `in`, `role` or `convenes`, and no border entry names the
  part. An organization part is exempt.

## Simulation

The simulator is the one agenticspec builds: a discrete-event simulation over simulated time,
deterministic per seed, with an exhaustive search and a hill-climbing search over the marked
places of the document. What an organization adds:

- **Load**: requests arrive per offering from its relationships.
- **Flow**: a request becomes work items that follow the steps of a process. Each step is done
  by a position that plays the step's role. A step between two positions is a hand-off.
- **Waiting**: a step that needs a decision waits for the decider: for a position's answer
  delay, or for the next meeting of the forum that settles the class. A stuck item escalates
  along its path after the path's time limit.
- **Capacity**: a position has its seats times the share of its holders. A holder works one
  item at a time.
- **Holders**: a human and an agent differ in speed, cost, error rate and hours. An agent's
  output that needs its accountable position's approval waits for that position.
- **Behaviour profile** (`orgspec-profile/1`): the distributions the specification cannot give:
  step durations per role, error and rework shares, answer delays, hours present, prices.
- **Faults**: a holder away, a position vacant, a demand surge, a forum skipped.

What the search may vary: seats, the agent share of a position, who decides a class, a
delegation's limit, a forum's cadence, an escalation's time limit, the unit a position sits in.

Limit: a simulation scores waiting, load, cost and reach. It does not score whether a decision
was right.

## KPIs per block

Every block has KPIs of three kinds, as in agenticspec:

- **Correctness**: a count that must be 0. The structural rules above, and rules of a run:
  decisions taken without the right, spend above a delegation's limit, work accepted by a
  position that plays no role of the step.
- **Completeness**: a share that should be 1: positions filled, subjects with an accountable
  position, key results with a current value, obligations with a control.
- **Quality**: a number with a direction: lead time, decision latency, the share of time work
  waits at a hand-off, span of control, depth of the reporting tree, subjects whose only
  competent holder is one party, commitments broken, cost per fulfilled request.

A posture changes directions and weights: an organization that explores accepts idle capacity
that an organization that operates does not.

The catalogue is `kpis/catalogue.yaml`, format `orgspec-kpis/1`. `scripts/check-kpis.sh` holds
it against the blocks and the events they declare.

## Repository layout

```text
blocks/                 the ESS system `org`: one directory per layer, one file per block
blocks/scenarios/       hand-written scenarios, run on the ESS interpreter
types/                  public organization types (orgspec-type/1)
organizations/          example designs (orgspec-organization/1), all invented
organizations/broken/   one design per rule, valid except for that rule
kpis/                   the KPI catalogue
design/                 this document, organization-types.md
scripts/                check-layers.sh, check-kpis.sh, check-public.sh, check-broken.sh,
                        check-blocks-conform.sh, blocks-digest.sh, their fixtures and tests
crates/orgspec/         the checker: `orgspec check`, `orgspec type check`
```

Nothing in this repository describes a real organization. A real organization's type, design
and records live in a repository of its own; see `design/organization-types.md` § "What stays
private".

## What ESS lacks here

Measured with `ess 0.99.2`. Each place carries a comment that starts `# UNMAPPED:` in the block
file, with the code ESS printed and what was taken instead.

- No arithmetic in a guard: spend is sent as a running total, and a delegation's limit is
  compared against an amount the command carries.
- No guard over a chain of rows: a reporting cycle, a cycle of parent units or parent
  organizations, the chain of `answers_to` and the cover of a unit's sub-units are checked by
  `orgspec check`, not by a block's command.
- A guard reads no variant of a union and compares no two union values (`ESS-COMMAND-002`,
  `ESS-COMMAND-003`). So the decider of a decision right is a struct with two optional fields,
  a channel's participant is two fields, accountability is keyed by its subject, and a unit that
  makes an agreement with itself is not refused.
- A guard compares no field of the subject with a field of a related row (`ESS-COMMAND-003`).
  So a unit under a parent of another organization, and a position whose role and unit belong
  to two organizations, are not refused.
- A view filters on no union and on no optional parameter (`ESS-VIEW-002`, `ESS-SYNTH-005`): the
  views publish such a field and the reader filters.
- No element of a list at a stored index, no time plus duration, no order on durations
  (`ESS-SPEC-012`, `ESS-COMMAND-002`): the wait of an escalation level and the cadence of a forum
  are stored and not held.
- Some rules validate and get no synthesized scenario (`ESS-SYNTH-001`, `-003`, `-004`, `-008`):
  a self-reference, a state reached only through a guarded step of another row, a command that
  addresses a subject and guards on a row it names. Each needs a hand-written scenario under
  `blocks/scenarios/`.
- Synthesis writes 25 scenarios that the interpreter refuses (`ESS-CF-OUTCOME`). A command that
  reads a related row through a stored field is arranged with the compared inputs left at their
  defaults: the chosen option is no member of the stored options, the instant is before the due
  instant, the position is not the one the work item is at. The three guards stay.
  `blocks/known-failing.txt` names the 25 scenarios under their cause, seven hand-written
  scenarios under `blocks/scenarios/` reach the same outcomes, and
  `scripts/check-blocks-conform.sh` fails on a failure that is not listed and on a listed
  scenario that passes. Not held yet: a floor for the number of scenarios.
- No import: a consumer and an organization type's extension copy the blocks and prove the
  copy by digest.

## Build order

1. Blocks: 30 domains in 13 layers, validated, layer-checked, with hand-written scenarios.
2. Types and the checker: `orgspec-type/1`, `orgspec check`, the public types, the examples.
3. KPI catalogue.
4. Lowering a design to commands of `org.design`, run on the ESS interpreter.
5. Simulator and search, on the crates agenticspec builds; behaviour profile; experiments.
6. Docs site: every block by layer with its KPIs, every type, every example.
7. Planned blocks.
8. Lifting an observed organization into a design and a run.
