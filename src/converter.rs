use crate::game_data::{EventData, EventOption, EventOutcome};
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// --- Configuration ---
const EVENTS_CSV_PATH: &str = "data/events.csv";
const OPTIONS_CSV_PATH: &str = "data/event_options.csv";
const JSON_OUTPUT_PATH: &str = "src/events.json";
// ---------------------

/// Represents a row in the `events.csv` file.
#[derive(Debug, Deserialize)]
struct EventCsvRow {
    event_id: String,
    title: String,
    description: String,
    min_tier: usize,
    max_tier: usize,
    is_generic: bool,
    life_stage: usize, // NEW: Added life_stage
}

/// Represents a row in the `event_options.csv` file.
#[derive(Debug, Deserialize)]
struct OptionCsvRow {
    event_id: String,
    text: String,
    
    // Success Outcome
    #[serde(default)]
    scs_change: i32,
    #[serde(default)]
    finance_change: i32,
    #[serde(default)]
    career_level_change: i32,
    #[serde(default)]
    guanxi_family_change: i32,
    #[serde(default)]
    guanxi_network_change: i32,
    #[serde(default)]
    guanxi_party_change: i32,
    
    // Requirements
    #[serde(default)]
    req_guanxi_family: u32,
    #[serde(default)]
    req_guanxi_network: u32,
    #[serde(default)]
    req_guanxi_party: u32,
    
    // Risk & Failure Outcome
    #[serde(default)]
    risk_chance: u8,
    #[serde(default)]
    success_result_text: String,
    #[serde(default)]
    failure_result_text: String,
    #[serde(default)]
    fail_scs_change: i32,
    #[serde(default)]
    fail_finance_change: i32,
    #[serde(default)]
    fail_career_level_change: i32,
    #[serde(default)]
    fail_guanxi_family_change: i32,
    #[serde(default)]
    fail_guanxi_network_change: i32,
    #[serde(default)]
    fail_guanxi_party_change: i32,
}

/// Helper to build an EventData from a CSV row.
fn create_event_from_row(row: EventCsvRow) -> EventData {
    EventData {
        title: row.title,
        description: row.description,
        min_tier: row.min_tier,
        max_tier: row.max_tier,
        is_generic: row.is_generic,
        life_stage: row.life_stage, // NEW: Pass life_stage
        options: Vec::new(), // Will be populated from the other file
    }
}

/// Helper to build an EventOption from a CSV row.
fn create_option_from_row(row: OptionCsvRow) -> EventOption {
    let success_outcome = EventOutcome {
        scs_change: row.scs_change,
        finance_change: row.finance_change,
        career_level_change: row.career_level_change,
        guanxi_family_change: row.guanxi_family_change,
        guanxi_network_change: row.guanxi_network_change,
        guanxi_party_change: row.guanxi_party_change,
    };

    let mut requirements = HashMap::new();
    if row.req_guanxi_family > 0 {
        requirements.insert("guanxi_family".to_string(), row.req_guanxi_family);
    }
    if row.req_guanxi_network > 0 {
        requirements.insert("guanxi_network".to_string(), row.req_guanxi_network);
    }
    if row.req_guanxi_party > 0 {
        requirements.insert("guanxi_party".to_string(), row.req_guanxi_party);
    }

    let mut failure_outcome = None;
    if row.risk_chance > 0 {
        let outcome = EventOutcome {
            scs_change: row.fail_scs_change,
            finance_change: row.fail_finance_change,
            career_level_change: row.fail_career_level_change,
            guanxi_family_change: row.fail_guanxi_family_change,
            guanxi_network_change: row.fail_guanxi_network_change,
            guanxi_party_change: row.fail_guanxi_party_change,
        };
        // Only set the failure outcome if it's actually different from success
        if outcome != success_outcome || !row.failure_result_text.is_empty() {
             failure_outcome = Some(outcome);
        }
    }

    EventOption {
        text: row.text,
        requirements,
        risk_chance: row.risk_chance,
        success_outcome,
        success_result: row.success_result_text,
        failure_outcome,
        failure_result: row.failure_result_text,
    }
}

/// Main converter function, called from `main.rs`.
pub fn run_converter() -> Result<()> {
    let mut events = HashMap::<String, EventData>::new();

    // 1. Read all events
    let mut event_reader = csv::Reader::from_path(EVENTS_CSV_PATH)?;
    for result in event_reader.deserialize() {
        let row: EventCsvRow = result?;
        events.insert(row.event_id.clone(), create_event_from_row(row));
    }

    // 2. Read all options and attach them to their events
    let mut option_reader = csv::Reader::from_path(OPTIONS_CSV_PATH)?;
    for result in option_reader.deserialize() {
        let row: OptionCsvRow = result?;
        if let Some(event) = events.get_mut(&row.event_id) {
            event.options.push(create_option_from_row(row));
        } else {
            eprintln!(
                "Warning: Option found for non-existent event_id {}",
                row.event_id
            );
        }
    }

    // 3. Convert the events HashMap to a Vec for the final JSON
    let final_event_list: Vec<EventData> = events.into_values().collect();

    // 4. Write the final JSON file
    let json_string = serde_json::to_string_pretty(&final_event_list)?;
    fs::write(JSON_OUTPUT_PATH, json_string)?;

    Ok(())
}