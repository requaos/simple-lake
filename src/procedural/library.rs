use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventDomain {
    Family,
    Work,
    Public,
    Party,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChoiceType {
    Conform,
    Resist,
    Manipulate,
    Ignore,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StatProfile {
    #[serde(default)]
    pub scs_change: i32,
    #[serde(default)]
    pub finance_change: i32,
    #[serde(default)]
    pub career_level_change: i32,
    #[serde(default)]
    pub guanxi_family_change: i32,
    #[serde(default)]
    pub guanxi_network_change: i32,
    #[serde(default)]
    pub guanxi_party_change: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NarrativeFragments {
    pub openings: Vec<String>,
    pub conflicts: Vec<String>,
    pub stakes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChoiceArchetype {
    pub archetype: ChoiceType,
    pub text_fragments: Vec<String>,
    #[serde(flatten)]
    pub base_stats: StatProfile,
    #[serde(default)]
    pub risk_modifier: i8,
    #[serde(default)]
    pub requirements: HashMap<String, u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SituationTemplate {
    pub id: String,
    pub domain: EventDomain,
    pub tier_min: usize,
    pub tier_max: usize,
    pub life_stage_min: usize,
    pub life_stage_max: usize,
    pub severity: Severity,
    pub base_risk: u8,
    pub fragments: NarrativeFragments,
    pub choices: Vec<ChoiceArchetype>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariableLibraries {
    pub colleague_descriptors: HashMap<String, Vec<String>>,
    pub excuse_library: Vec<String>,
    pub relationship_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SituationLibrary {
    pub by_domain: HashMap<EventDomain, Vec<SituationTemplate>>,
    pub variables: VariableLibraries,
}

impl SituationLibrary {
    pub fn from_embedded_configs() -> Result<Self> {
        // Load embedded TOML files
        let work_toml = include_str!("../../data/procedural/work_events.toml");
        let family_toml = include_str!("../../data/procedural/family_events.toml");
        let public_toml = include_str!("../../data/procedural/public_events.toml");
        let party_toml = include_str!("../../data/procedural/party_events.toml");
        let variables_toml = include_str!("../../data/procedural/variables.toml");

        // Parse situations
        let work_config: SituationConfig =
            toml::from_str(work_toml).context("Failed to parse work_events.toml")?;
        let family_config: SituationConfig =
            toml::from_str(family_toml).context("Failed to parse family_events.toml")?;
        let public_config: SituationConfig =
            toml::from_str(public_toml).context("Failed to parse public_events.toml")?;
        let party_config: SituationConfig =
            toml::from_str(party_toml).context("Failed to parse party_events.toml")?;

        // Parse variables
        let variables: VariableLibraries =
            toml::from_str(variables_toml).context("Failed to parse variables.toml")?;

        // Build by_domain HashMap
        let mut by_domain = HashMap::new();
        by_domain.insert(EventDomain::Work, work_config.situations);
        by_domain.insert(EventDomain::Family, family_config.situations);
        by_domain.insert(EventDomain::Public, public_config.situations);
        by_domain.insert(EventDomain::Party, party_config.situations);

        Ok(Self {
            by_domain,
            variables,
        })
    }
}

// Helper struct for TOML deserialization
#[derive(Debug, Deserialize)]
struct SituationConfig {
    situations: Vec<SituationTemplate>,
}

impl EventDomain {
    pub fn as_str(&self) -> &str {
        match self {
            EventDomain::Family => "Family",
            EventDomain::Work => "Work",
            EventDomain::Public => "Public",
            EventDomain::Party => "Party",
        }
    }
}

impl ChoiceType {
    pub fn as_str(&self) -> &str {
        match self {
            ChoiceType::Conform => "conform",
            ChoiceType::Resist => "resist",
            ChoiceType::Manipulate => "manipulate",
            ChoiceType::Ignore => "ignore",
        }
    }
}
