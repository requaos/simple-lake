# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based game built with `eframe`/`egui` that simulates a social credit system progression game. Players navigate through tiers and life stages, encountering events and making choices that affect various stats (Social Credit Score, Finances, Career, Guanxi).

**Tech Stack:**
- Rust (edition 2024)
- eframe/egui 0.33 for GUI
- CSV data files compiled to JSON at build time

## Build & Run Commands

### Running the game
```bash
cargo run
```

### Converting CSV event data to JSON
The game uses `data/events.csv` and `data/event_options.csv` to define game content. These must be converted to `src/events.json` before running:
```bash
cargo run -- --convert
```

### Standard Rust commands
```bash
cargo build          # Build the project
cargo check          # Check for compilation errors
cargo clippy         # Run lints
cargo fmt            # Format code
```

## Architecture

### Module Structure
- `main.rs` - Entry point, handles `--convert` flag and initializes the game
- `app.rs` - Main game loop (`eframe::App` trait), UI panels, event handling, player state updates
- `game_data.rs` - Event system data structures and event generation logic
- `converter.rs` - CSV → JSON conversion for event data
- `lotus_widget.rs` - Custom egui widget for the lotus flower game board visualization
- `procedural/` - **NEW:** Procedural event generation system
  - `mod.rs` - Module exports
  - `library.rs` - Core data structures (SituationTemplate, EventDomain, etc.)
  - `generator.rs` - Main event generation engine
  - `text_assembly.rs` - Madlibs-style text generation
  - `stat_calculator.rs` - Context-driven stat calculation
  - `risk_calculator.rs` - Hybrid risk calculation

### Game State (`LotusApp`)
All game state lives in a single struct:
- Player stats: tier, petal position, age, life_stage, social_credit_score, finances, career_level, guanxi_{family,network,party}
- Event database: Pre-loaded from `src/events.json` (embedded at compile time with `include_str!`)
- Event index: Pre-computed HashMap for fast event lookups by (life_stage, tier)
- **Procedural system:** situation_library, recent_event_domains (VecDeque, last 15), encounter_history (HashSet), event_counter, encounter_map
- UI state: current_event, floating_texts, history log

### Event System

**Event Generation (Procedural-First):**
The game now uses a **hybrid event system** that attempts procedural generation first, then falls back to handcrafted events:

1. **Procedural Generation** (`src/procedural/generator.rs`):
   - Generates events on-the-fly from TOML situation templates
   - Context-aware filtering (no repeated situations within 30 events, no repeated domains within 2 events)
   - 10% wildcard probability for surprise events
   - Madlibs-style text assembly with tier-appropriate variable substitution
   - Context-driven stat calculation (tier × severity × random variance)
   - Hybrid risk calculation (base risk + requirement gaps + choice modifiers)

2. **Handcrafted Event Fallback** (`src/game_data.rs`):
   - If procedural generation fails (no valid situations), falls back to CSV-based events
   - Try tier-specific events for current life stage
   - Fall back to generic events for current life stage
   - Fall back to generic events from previous life stages
   - Final fallback: error event

**Event Metadata:**
- Events have optional `procedural_id` and `procedural_domain` fields
- Context tracking updates after procedural events resolve (in `app.rs`)

**Event Options:**
- Requirements (guanxi levels, career level)
- Risk/reward mechanics (success_outcome vs failure_outcome)
- Multiple stat changes per choice

### Procedural Event Configuration

**TOML Files** (`data/procedural/`):
- `work_events.toml`, `family_events.toml`, `public_events.toml`, `party_events.toml`
- `variables.toml` - Variable libraries for madlibs substitution

**Situation Template Format:**
```toml
[[situations]]
id = "work_promotion_passed_over"
domain = "work"
tier_min = 1
tier_max = 3
life_stage_min = 2
life_stage_max = 4
severity = "medium"  # low/medium/high
base_risk = 20

[situations.fragments]
openings = ["Opening text with {variables}..."]
conflicts = ["Conflict text..."]
stakes = ["Stakes text..."]

[[situations.choices]]
archetype = "conform"  # conform/resist/manipulate/ignore
text_fragments = ["Accept it gracefully", "Move on"]
base_scs = 15
base_finance = -10
risk_modifier = -10
requirements = { career_level = 2 }
```

**Loading:**
- TOML files are embedded at compile time with `include_str!()`
- Parsed into `SituationLibrary` at startup
- Organized by domain in HashMap for fast lookups

### Data Flow

**Procedural Events:**
1. Player lands on petal → `app.rs` calls `generate_event()`
2. `generate_event()` attempts `generate_procedural_event(player_state, rng)`
3. Generator filters situations by tier/stage/history/domain
4. Weighted random selection (prefer exact tier/stage matches)
5. Madlibs assembly: select random fragments, substitute variables
6. Calculate stats (context-driven) and risk (hybrid)
7. Build `EventData` with `procedural_id` and `procedural_domain`
8. Player resolves event → `update_event_context()` tracks domain and situation ID

**Handcrafted Events (Fallback):**
1. CSV files in `data/` define events and options
2. `cargo run -- --convert` merges CSVs → `src/events.json`
3. At runtime, JSON is embedded and deserialized into `Vec<EventData>`
4. Event index is pre-computed for O(1) lookups
5. If procedural generation returns None, select from index

### Key Patterns
- **Asset embedding**: Both `events.json` and TOML configs embedded with `include_str!()` for single-binary distribution
- **Pre-computation**: Event index and situation library built at startup to avoid linear searches
- **Requirement filtering**: Event options are filtered dynamically based on player stats
- **Context tracking**: Bounded memory (VecDeque for last 15 domains, HashSet for last 30 situations) prevents repetition
- **Procedural-first design**: Attempts procedural generation, falls back to handcrafted on failure
- **Madlibs text generation**: Random fragment selection + tier-appropriate variable substitution
- **Context-driven stats**: `tier_multiplier × severity_multiplier × random_variance` applied to base stats
- **Hybrid risk**: `base_risk + (requirement_gap × 5) + choice_modifier` clamped to 0-95
- **Animation**: egui's `animate_value_with_time()` for smooth transitions (player token, petal hover, glow effects)

## CSV Data Format

### events.csv columns
- event_id, title, description, min_tier, max_tier, is_generic, life_stage

### event_options.csv columns
- event_id, text
- Success: scs_change, finance_change, career_level_change, guanxi_{family,network,party}_change, success_result_text
- Requirements: req_guanxi_{family,network,party}
- Risk: risk_chance (0-100), failure_result_text, fail_* variants of stat changes

## Social Credit Tier System

Tier thresholds (defined in `app.rs`):
- Tier D: ≤ 199
- Tier C: 200-399
- Tier B: 400-749
- Tier A: 750-999
- Tier A+: ≥ 1000

## Life Stages

Age-based progression (defined in `app.rs`):
1. Youth (18-25)
2. Early Career (26-40)
3. Mid-Career (41-55)
4. Seniority (56+)

Age increments when moving backwards across petal 0 boundary.

## Important Details

- The game board has 5 tiers × 13 petals per tier
- Special petals: 0 (birthday/age-up), 4 & 8 (SCS review checkpoints)
- Player token animates smoothly between petals
- Floating text appears for stat changes (fades out over 2 seconds)
- History log tracks all events with player age prefix
