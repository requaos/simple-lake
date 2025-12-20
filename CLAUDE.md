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

### Game State (`LotusApp`)
All game state lives in a single struct:
- Player stats: tier, petal position, age, life_stage, social_credit_score, finances, career_level, guanxi_{family,network,party}
- Event database: Pre-loaded from `src/events.json` (embedded at compile time with `include_str!`)
- Event index: Pre-computed HashMap for fast event lookups by (life_stage, tier)
- UI state: current_event, floating_texts, history log

### Event System
Events are selected based on player's current tier and life stage:
1. Try tier-specific events for current life stage
2. Fall back to generic events for current life stage
3. Fall back to generic events from previous life stages
4. Final fallback: error event

Event options can have:
- Requirements (guanxi levels, career level)
- Risk/reward mechanics (success_outcome vs failure_outcome)
- Multiple stat changes per choice

### Data Flow
1. CSV files in `data/` define events and options
2. `cargo run -- --convert` merges CSVs → `src/events.json`
3. At runtime, JSON is embedded and deserialized into `Vec<EventData>`
4. Event index is pre-computed for O(1) lookups
5. `generate_event()` selects appropriate events based on player state

### Key Patterns
- **Asset embedding**: `events.json` is embedded with `include_str!()` for single-binary distribution
- **Pre-computation**: Event index built at startup to avoid linear searches
- **Requirement filtering**: Event options are filtered dynamically based on player stats
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
