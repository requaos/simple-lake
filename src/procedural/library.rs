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
    // Work variables
    pub colleague_descriptors: HashMap<String, Vec<String>>,
    pub excuse_library: Vec<String>,
    #[serde(default)]
    pub work_time: Vec<String>,
    #[serde(default)]
    pub work_colleague: Vec<String>,
    #[serde(default)]
    pub work_day: Vec<String>,
    #[serde(default)]
    pub work_obligation: Vec<String>,
    #[serde(default)]
    pub work_record: Vec<String>,
    #[serde(default)]
    pub overtime_period: Vec<String>,
    #[serde(default)]
    pub work_project: Vec<String>,
    #[serde(default)]
    pub safety_violation: Vec<String>,
    #[serde(default)]
    pub political_metric: Vec<String>,
    #[serde(default)]
    pub monitoring_target: Vec<String>,
    #[serde(default)]
    pub political_team_activity: Vec<String>,
    #[serde(default)]
    pub work_mistake: Vec<String>,
    #[serde(default)]
    pub bribe_amount: Vec<String>,
    #[serde(default)]
    pub work_decision: Vec<String>,

    // Family variables
    pub relationship_types: Vec<String>,
    #[serde(default)]
    pub parent_type: Vec<String>,
    #[serde(default)]
    pub sibling_type: Vec<String>,
    #[serde(default)]
    pub relative_type: Vec<String>,
    #[serde(default)]
    pub small_amount: Vec<String>,
    #[serde(default)]
    pub medium_amount: Vec<String>,
    #[serde(default)]
    pub large_amount: Vec<String>,
    #[serde(default)]
    pub time_period: Vec<String>,
    #[serde(default)]
    pub authority_figure: Vec<String>,
    #[serde(default)]
    pub infraction: Vec<String>,
    #[serde(default)]
    pub unpractical_subject: Vec<String>,
    #[serde(default)]
    pub practical_subject: Vec<String>,
    #[serde(default)]
    pub personal_topic: Vec<String>,
    #[serde(default)]
    pub successful_relative: Vec<String>,
    #[serde(default)]
    pub unsuitable_match: Vec<String>,

    // Party variables
    #[serde(default)]
    pub political_topic: Vec<String>,
    #[serde(default)]
    pub day_time: Vec<String>,
    #[serde(default)]
    pub time_duration: Vec<String>,
    #[serde(default)]
    pub party_observer: Vec<String>,
    #[serde(default)]
    pub membership_level: Vec<String>,
    #[serde(default)]
    pub party_official: Vec<String>,
    #[serde(default)]
    pub controversial_topic: Vec<String>,
    #[serde(default)]
    pub denouncement_target: Vec<String>,
    #[serde(default)]
    pub political_crime: Vec<String>,
    #[serde(default)]
    pub volunteer_activity: Vec<String>,
    #[serde(default)]
    pub party_elite: Vec<String>,
    #[serde(default)]
    pub favor_request: Vec<String>,
    #[serde(default)]
    pub propaganda_campaign: Vec<String>,
    #[serde(default)]
    pub propaganda_activity: Vec<String>,

    // Public variables
    #[serde(default)]
    pub stranger_type: Vec<String>,
    #[serde(default)]
    pub small_favor: Vec<String>,
    #[serde(default)]
    pub public_place: Vec<String>,
    #[serde(default)]
    pub appointment_type: Vec<String>,
    #[serde(default)]
    pub public_violation: Vec<String>,
    #[serde(default)]
    pub violation_perpetrator: Vec<String>,
    #[serde(default)]
    pub public_service: Vec<String>,
    #[serde(default)]
    pub queue_jumper: Vec<String>,
    #[serde(default)]
    pub public_transport: Vec<String>,
    #[serde(default)]
    pub seat_requester: Vec<String>,
    #[serde(default)]
    pub suspicious_behavior: Vec<String>,
    #[serde(default)]
    pub survey_topic: Vec<String>,
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
