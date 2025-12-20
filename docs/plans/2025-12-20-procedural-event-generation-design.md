# Procedural Event Generation System - Design

**Date:** 2025-12-20
**Status:** Approved for Implementation

## Overview

Replace the current CSV-based handcrafted event system with a fully procedural event generator capable of creating infinite contextually-relevant events during gameplay. The system will generate events on-the-fly based on player state, ensuring variety while avoiding repetition.

## Goals

- **Avoid repetition:** Prevent players from seeing the same events multiple times
- **Replayability:** Each playthrough feels different
- **Depth per tier/stage:** Rich variety for all tier and life stage combinations
- **Procedural variety:** Events feel contextually generated based on player state

## Design Principles

- **Fully procedural:** Events generated at runtime, not pre-generated
- **Component library:** Build from reusable situations and choice archetypes
- **Moderate context:** Track recent history to prevent repetition without deep narrative state
- **Preserve complexity:** Generated events support full feature set (requirements, risk/reward, multiple outcomes)
- **Controlled randomness:** Most events fit expectations, occasional wild cards for surprise
- **Fallback safety:** Handcrafted events remain as fallback during development

## Architecture

### Four-Layer System

#### 1. Component Library Layer
- Hierarchical organization by domain (Family, Work, Public, Party)
- Situation templates with pre-paired choice archetypes
- Stored in external TOML config files, loaded at startup
- Tier/stage filtering built into template metadata

#### 2. Context Tracking Layer
- Lightweight history tracking in `LotusApp`
- Recent event domain sequence (last 10-15 events)
- Situation encounter history (last 30 events by ID)
- Event counter for aging out old history

#### 3. Generation Engine Layer
- Situation selection with context-aware filtering
- Choice assembly with requirement validation
- Madlibs-style text generation
- Context-driven stat calculation
- Hybrid risk computation

#### 4. Fallback Layer
- Existing handcrafted event system preserved
- Procedural generation attempted first
- Falls back on failure or during incomplete development

## Component Library Structure

### External Config Format

Domain-specific TOML files: `work_events.toml`, `family_events.toml`, `public_events.toml`, `party_events.toml`

**Situation Template:**
```toml
[[situations]]
id = "work_promotion_passed_over"
domain = "Work"
tier_min = 1
tier_max = 3
life_stage_min = 2
life_stage_max = 4
severity = "medium"
base_risk = 20

[situations.fragments]
openings = [
  "A promotion you were qualified for goes to {colleague_descriptor}.",
  "You watch as {colleague_descriptor} gets the promotion you deserved.",
]
conflicts = [
  "Your boss says it's '{excuse}'.",
  "Management cites '{excuse}' as the reason.",
]
stakes = [
  "This could affect your career trajectory.",
  "Your colleagues are watching how you react.",
]

[[situations.choices]]
archetype = "conform"
text_fragments = ["Accept it gracefully", "Congratulate your colleague"]
base_scs = 15
base_finance = -10
# ... other stat profiles ...
risk_modifier = -10

[[situations.choices]]
archetype = "resist"
text_fragments = ["Demand an explanation", "File a formal complaint"]
base_scs = -20
base_finance = 0
risk_modifier = 30
requirements = { career_level = 2 }
# ... other choices ...
```

### Runtime Data Structures

```rust
struct SituationTemplate {
    id: String,
    domain: EventDomain,
    tier_min: usize,
    tier_max: usize,
    life_stage_min: usize,
    life_stage_max: usize,
    severity: Severity,  // low/medium/high
    base_risk: u8,
    fragments: NarrativeFragments,
    choices: Vec<ChoiceArchetype>,
}

struct NarrativeFragments {
    openings: Vec<String>,
    conflicts: Vec<String>,
    stakes: Vec<String>,
}

struct ChoiceArchetype {
    archetype: ChoiceType,  // conform/resist/manipulate/ignore
    text_fragments: Vec<String>,
    base_stats: StatProfile,
    risk_modifier: i8,
    requirements: HashMap<String, u32>,
}

struct SituationLibrary {
    by_domain: HashMap<EventDomain, Vec<SituationTemplate>>,
    variable_libraries: VariableLibraries,  // for madlibs substitution
}
```

## Context Tracking

### New Fields in `LotusApp`

```rust
pub struct LotusApp {
    // ... existing fields ...

    // Procedural event system
    situation_library: SituationLibrary,

    // Context tracking
    recent_event_domains: VecDeque<EventDomain>,  // Last 15
    encounter_history: HashSet<String>,  // Situation IDs, last 30 events
    event_counter: usize,
    encounter_map: HashMap<String, usize>,  // situation_id -> event_counter
}
```

### Tracking Logic

- After each event: push domain to `recent_event_domains` (max 15)
- Record situation ID in `encounter_history` and `encounter_map`
- Increment `event_counter`
- Periodically clean old entries (> 30 events ago) from maps

### Anti-Repetition Rules

- Filter out situations if domain appears in last 2 events
- Filter out situations if ID is in `encounter_history`
- 10% wild card probability: ignore domain filter for surprise events

## Generation Engine

### Core Function

```rust
pub fn generate_procedural_event(
    player_state: &LotusApp,
    situation_library: &SituationLibrary,
    rng: &mut impl Rng,
) -> Option<EventData>
```

### Generation Steps

**1. Situation Selection:**
- Filter by tier: `player_tier ± 1` (allow some variance)
- Filter by life_stage: current or previous stage
- Apply context filters: exclude recent domains (last 2) and encountered IDs
- 10% wild card: skip domain filter
- Weight candidates by tier/stage match quality
- Weighted random selection

**2. Choice Assembly:**
- Iterate through situation's pre-paired choices
- Filter by requirements vs player stats
- Ensure ≥1 choice available (regenerate if all locked)
- Keep 2-4 choices

**3. Text Generation:**
- Select random fragments from arrays
- Variable substitution (colleague_descriptor, excuse, etc.)
- Assemble: `opening + conflict + stakes` = description
- Generate choice text from archetype fragments

**4. Stat Calculation (Context-Driven):**
```rust
tier_multiplier = (player_tier + 1) as f32 * 1.5
severity_multiplier = match severity {
    Severity::Low => 0.5,
    Severity::Medium => 1.0,
    Severity::High => 2.0,
}
random_variance = rng.gen_range(0.8..1.2)

scs_change = (base_scs * tier_multiplier * severity_multiplier * random_variance) as i32
```

**5. Risk Calculation (Hybrid):**
```rust
// Base from situation
base_risk = situation.base_risk

// Player state adjustment
requirement_gap = max(0, requirement - player_stat)
risk_adjustment = requirement_gap * 5

// Choice modifier
archetype_modifier = choice.risk_modifier

final_risk = clamp(base_risk + risk_adjustment + archetype_modifier, 0, 95)
```

Failure outcomes use inverted/amplified stat profiles.

## Madlibs Text Assembly

### Variable Libraries

Loaded from config, organized by tier:
```rust
struct VariableLibraries {
    colleague_descriptors: HashMap<usize, Vec<String>>,  // tier -> descriptors
    excuse_library: Vec<String>,
    relationship_types: Vec<String>,
    // ... more as needed
}
```

### Assembly Process

```rust
// Select fragments
opening = random_from(situation.fragments.openings)
conflict = random_from(situation.fragments.conflicts)
stakes = random_from(situation.fragments.stakes)

// Variable substitution
colleague_desc = weighted_random(tier_descriptors[player_tier])
excuse = random_from(excuse_library)

text = opening.replace("{colleague_descriptor}", colleague_desc)
            .replace("{excuse}", excuse)
            // ... more substitutions

description = format!("{} {} {}", text, conflict, stakes)
```

## Fallback Integration

### Modified `generate_event()`

```rust
pub fn generate_event(player_state: &LotusApp) -> EventData {
    let mut rng = rand::rng();

    // Attempt procedural generation
    if let Some(event) = generate_procedural_event(
        player_state,
        &player_state.situation_library,
        &mut rng
    ) {
        return event;
    }

    // Fallback to handcrafted
    // ... existing event_database logic ...
}
```

### Fallback Triggers

- No situations pass filters
- All choices filtered by requirements
- TOML parsing errors (logs warning, continues)
- Generator panics (caught, logged)

## Data Flow

1. Player lands on petal → `app.rs` calls `generate_event()`
2. Check context: read `recent_event_domains`, `encounter_history`
3. Filter `situation_library` → weighted selection
4. Retrieve pre-paired choices
5. Evaluate requirements, calculate stats/risk
6. Assemble text via madlibs
7. Return `EventData` → UI displays modal
8. Player chooses → outcome applied → context updated

## Implementation Notes

### Startup Changes (`main.rs`)

```rust
// Load situation library from embedded TOML
let situation_library = SituationLibrary::from_embedded_configs()?;

// Initialize in LotusApp
LotusApp {
    // ... existing fields ...
    situation_library,
    recent_event_domains: VecDeque::new(),
    encounter_history: HashSet::new(),
    event_counter: 0,
    encounter_map: HashMap::new(),
}
```

### New Module Structure

```
src/
  procedural/
    mod.rs              # Public interface
    library.rs          # SituationLibrary, loading logic
    generator.rs        # generate_procedural_event()
    text_assembly.rs    # Madlibs engine
    stat_calculator.rs  # Context-driven stat formulas
    risk_calculator.rs  # Hybrid risk system
```

### Config Files (Embedded)

```
data/procedural/
  work_events.toml
  family_events.toml
  public_events.toml
  party_events.toml
  variables.toml  # Descriptor/excuse libraries
```

Embed with `include_str!()` like current `events.json`.

## Testing Strategy

1. **Unit tests:** Stat calculation formulas, risk calculation, filtering logic
2. **Integration tests:** Full generation with mock library, verify EventData output
3. **Manual testing:** Play multiple sessions, verify no immediate repetition
4. **Fallback testing:** Delete TOML files, verify handcrafted events still work

## Future Extensions (Out of Scope)

- Named NPCs with persistent relationships
- Branching storylines across events
- Player choice pattern analysis for tailored events
- LLM integration for flavor text variation
- Save/load encounter history across sessions

## Success Criteria

- Game runs with procedural events enabled
- No repeated situations within 30 events
- No repeated domains within 2 events
- Events feel contextually appropriate to tier/stage
- Fallback to handcrafted events works reliably
- Performance: event generation < 5ms on modest hardware
