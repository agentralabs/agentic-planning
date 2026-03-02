---
status: stable
---

# CLI Reference

## Global Options

```
agentic-planning [OPTIONS] <COMMAND>

Options:
  --file <PATH>        Planning state file
  --format <FORMAT>    Output format: text, json, table (default: text)
  --json               Shorthand for --format json
  -v, --verbose        Enable verbose output
```

---

## goal — Manage planning goals

### goal create

```
agentic-planning goal create <TITLE> [OPTIONS]

Arguments:
  <TITLE>                    Goal title

Options:
  -d, --description <DESC>   Goal description
  -i, --intention <INTENT>   Why this goal matters
  -p, --priority <PRIORITY>  critical|high|medium|low|someday
  --deadline <DATE>           Target completion date
  --parent <ID>               Parent goal UUID
  --tags <TAGS>               Comma-separated tags
  --activate                  Immediately activate
```

### goal list

```
agentic-planning goal list [OPTIONS]

Options:
  -s, --status <STATUS>      Filter by status
  -p, --priority <PRIORITY>  Filter by priority
  --tag <TAG>                 Filter by tag
  --active                    Show only active goals
  --blocked                   Show only blocked goals
  --urgent                    Show urgent goals
  -n, --limit <N>             Limit results
```

### goal show

```
agentic-planning goal show <ID> [--full]
```

### goal activate / complete / pause / resume / abandon

```
agentic-planning goal activate <ID>
agentic-planning goal complete <ID> [-n, --note <NOTE>]
agentic-planning goal pause <ID> [-r, --reason <REASON>]
agentic-planning goal resume <ID>
agentic-planning goal abandon <ID> <REASON>
```

### goal progress

```
agentic-planning goal progress <ID> <PERCENTAGE> [-n, --note <NOTE>]
```

### goal block / unblock

```
agentic-planning goal block <ID> <BLOCKER> [-s, --severity <0.0-1.0>]
agentic-planning goal unblock <ID> <BLOCKER_ID> <RESOLUTION>
```

### goal decompose / link / tree

```
agentic-planning goal decompose <ID> [--sub_goals <SPECS>]
agentic-planning goal link <GOAL_A> <GOAL_B> [-r, --relationship <TYPE>]
agentic-planning goal tree [ID]
```

### goal feelings / physics / dream / reincarnate

```
agentic-planning goal feelings <ID>
agentic-planning goal physics <ID>
agentic-planning goal dream <ID>
agentic-planning goal reincarnate <ID> [-t, --title <TITLE>] [--lessons <LESSONS>]
```

---

## decision — Manage decisions

```
agentic-planning decision create <QUESTION> [-c, --context <CTX>]
agentic-planning decision option <ID> <NAME> [-d, --description <DESC>] [--pros <PROS>] [--cons <CONS>]
agentic-planning decision crystallize <ID> <CHOSEN> [-r, --reasoning <REASONING>]
agentic-planning decision show <ID>
agentic-planning decision shadows <ID>
agentic-planning decision chain <ID>
agentic-planning decision archaeology <ARTIFACT>
agentic-planning decision prophecy <QUESTION> [--options <OPTIONS>]
agentic-planning decision regret <ID>
agentic-planning decision recrystallize <ID> <CHOSEN> <REASON>
```

---

## commitment — Track commitments

```
agentic-planning commitment create <PROMISE> <STAKEHOLDER> [OPTIONS]
  --due <DATE>                Due date
  --goal <GOAL_ID>            Linked goal
  --role <ROLE>               Stakeholder role (default: stakeholder)
  --importance <0.0-1.0>      Importance weight (default: 0.6)

agentic-planning commitment list
agentic-planning commitment show <ID>
agentic-planning commitment fulfill <ID> <HOW_DELIVERED>
agentic-planning commitment break <ID> <REASON>
agentic-planning commitment renegotiate <ID> <NEW_PROMISE> <REASON>
agentic-planning commitment entangle <ID> <COMMITMENT_B> [--entanglement_type <TYPE>] [--strength <0.0-1.0>]
agentic-planning commitment inventory
agentic-planning commitment due-soon [--within_days <N>]
agentic-planning commitment at-risk
```

---

## progress — Track execution momentum

```
agentic-planning progress momentum [--goal_id <ID>]
agentic-planning progress gravity
agentic-planning progress blockers
agentic-planning progress echoes
agentic-planning progress forecast <GOAL_ID>
agentic-planning progress velocity
agentic-planning progress trend
```

---

## singularity — Analyze convergence

```
agentic-planning singularity collapse
agentic-planning singularity position <GOAL_ID>
agentic-planning singularity path
agentic-planning singularity tensions
agentic-planning singularity themes
agentic-planning singularity center
agentic-planning singularity vision
```

---

## dream — Predictive insights

```
agentic-planning dream goal <ID>
agentic-planning dream collective [--federation_id <ID>]
agentic-planning dream interpret <DREAM_ID>
agentic-planning dream insights [--goal_id <ID>]
agentic-planning dream accuracy <DREAM_ID>
agentic-planning dream history [--goal_id <ID>]
```

---

## counterfactual — Explore alternatives

```
agentic-planning counterfactual project <DECISION_ID> <PATH_ID>
agentic-planning counterfactual compare <DECISION_ID> <PATH_A> <PATH_B>
agentic-planning counterfactual learn <DECISION_ID>
agentic-planning counterfactual timeline <DECISION_ID> [--path_id <ID>]
```

---

## chain — Decision causality

```
agentic-planning chain trace <DECISION_ID>
agentic-planning chain cascade <DECISION_ID>
agentic-planning chain roots <DECISION_ID>
agentic-planning chain leaves <DECISION_ID>
agentic-planning chain visualize <DECISION_ID>
```

---

## metamorphosis — Goal transformation

```
agentic-planning metamorphosis detect <GOAL_ID>
agentic-planning metamorphosis approve <GOAL_ID> [OPTIONS]
  --title <TITLE>              New title
  --description <DESC>         New description
  --change_type <TYPE>         expansion|contraction|pivot|refinement
  --factor <0.0-1.0>           Change magnitude
  --reason <REASON>            Reason
  --new_direction <DIR>        For pivot
  --clarification <TEXT>       For refinement

agentic-planning metamorphosis history <GOAL_ID>
agentic-planning metamorphosis predict <GOAL_ID>
agentic-planning metamorphosis stage <GOAL_ID>
```

---

## consensus — Multi-stakeholder alignment

```
agentic-planning consensus start <DECISION_ID> [--stakeholders <LIST>]
agentic-planning consensus round <DECISION_ID> <STAKEHOLDER> <POSITION>
agentic-planning consensus vote <DECISION_ID> <STAKEHOLDER> <OPTION>
agentic-planning consensus synthesize <DECISION_ID>
agentic-planning consensus status <DECISION_ID>
agentic-planning consensus crystallize <DECISION_ID> [--option <OPT>] [--rationale <TEXT>]
```

---

## federation — Multi-agent coordination

```
agentic-planning federation create <GOAL_ID> <AGENT_ID> [--coordinator <ID>]
agentic-planning federation join <FEDERATION_ID> <AGENT_ID>
agentic-planning federation sync <FEDERATION_ID>
agentic-planning federation handoff <FEDERATION_ID> <AGENT_ID>
agentic-planning federation status <FEDERATION_ID>
agentic-planning federation members <FEDERATION_ID>
agentic-planning federation list
```

---

## workspace — Planning contexts

```
agentic-planning workspace create <NAME> [--file <PATH>]
agentic-planning workspace switch <NAME>
agentic-planning workspace list
agentic-planning workspace compare <LEFT> <RIGHT>
agentic-planning workspace merge <SOURCE> <TARGET>
agentic-planning workspace delete <NAME>
```

---

## daemon — Background service

```
agentic-planning daemon start [--interval_secs <N>]
agentic-planning daemon stop
agentic-planning daemon status
agentic-planning daemon logs [-n, --lines <N>]
```

---

## Other commands

```
agentic-planning status                     Show overall planning status
agentic-planning context-log <INTENT> [--finding <TEXT>] [--topic <CATEGORY>]
agentic-planning serve [--mode stdio|http] [--port <PORT>]
agentic-planning version                    Show version
```
