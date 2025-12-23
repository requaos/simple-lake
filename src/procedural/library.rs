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
    pub wait_time: Vec<String>,
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

        // Log variable library statistics - ALL variables
        log::info!("=== Variable Library Loaded ===");
        log::info!("Work variables:");
        log::info!("  excuse_library: {} items", variables.excuse_library.len());
        log::info!("  work_time: {} items", variables.work_time.len());
        log::info!("  work_colleague: {} items", variables.work_colleague.len());
        log::info!("  work_day: {} items", variables.work_day.len());
        log::info!("  work_obligation: {} items", variables.work_obligation.len());
        log::info!("  work_record: {} items", variables.work_record.len());
        log::info!("  overtime_period: {} items", variables.overtime_period.len());
        log::info!("  work_project: {} items", variables.work_project.len());
        log::info!("  safety_violation: {} items", variables.safety_violation.len());
        log::info!("  political_metric: {} items", variables.political_metric.len());
        log::info!("  monitoring_target: {} items", variables.monitoring_target.len());
        log::info!("  political_team_activity: {} items", variables.political_team_activity.len());
        log::info!("  work_mistake: {} items", variables.work_mistake.len());
        log::info!("  bribe_amount: {} items", variables.bribe_amount.len());
        log::info!("  work_decision: {} items", variables.work_decision.len());

        log::info!("Family variables:");
        log::info!("  relationship_types: {} items", variables.relationship_types.len());
        log::info!("  parent_type: {} items", variables.parent_type.len());
        log::info!("  sibling_type: {} items", variables.sibling_type.len());
        log::info!("  relative_type: {} items", variables.relative_type.len());
        log::info!("  small_amount: {} items", variables.small_amount.len());
        log::info!("  medium_amount: {} items", variables.medium_amount.len());
        log::info!("  large_amount: {} items", variables.large_amount.len());
        log::info!("  time_period: {} items", variables.time_period.len());
        log::info!("  authority_figure: {} items", variables.authority_figure.len());
        log::info!("  infraction: {} items", variables.infraction.len());
        log::info!("  unpractical_subject: {} items", variables.unpractical_subject.len());
        log::info!("  practical_subject: {} items", variables.practical_subject.len());
        log::info!("  personal_topic: {} items", variables.personal_topic.len());
        log::info!("  successful_relative: {} items", variables.successful_relative.len());
        log::info!("  unsuitable_match: {} items", variables.unsuitable_match.len());

        log::info!("Party variables:");
        log::info!("  political_topic: {} items", variables.political_topic.len());
        log::info!("  day_time: {} items", variables.day_time.len());
        log::info!("  time_duration: {} items", variables.time_duration.len());
        log::info!("  party_observer: {} items", variables.party_observer.len());
        log::info!("  membership_level: {} items", variables.membership_level.len());
        log::info!("  party_official: {} items", variables.party_official.len());
        log::info!("  controversial_topic: {} items", variables.controversial_topic.len());
        log::info!("  denouncement_target: {} items", variables.denouncement_target.len());
        log::info!("  political_crime: {} items", variables.political_crime.len());
        log::info!("  volunteer_activity: {} items", variables.volunteer_activity.len());
        log::info!("  party_elite: {} items", variables.party_elite.len());
        log::info!("  favor_request: {} items", variables.favor_request.len());
        log::info!("  propaganda_campaign: {} items", variables.propaganda_campaign.len());
        log::info!("  propaganda_activity: {} items", variables.propaganda_activity.len());

        log::info!("Public variables:");
        log::info!("  wait_time: {} items", variables.wait_time.len());
        log::info!("  stranger_type: {} items", variables.stranger_type.len());
        log::info!("  small_favor: {} items", variables.small_favor.len());
        log::info!("  public_place: {} items", variables.public_place.len());
        log::info!("  appointment_type: {} items", variables.appointment_type.len());
        log::info!("  public_violation: {} items", variables.public_violation.len());
        log::info!("  violation_perpetrator: {} items", variables.violation_perpetrator.len());
        log::info!("  public_service: {} items", variables.public_service.len());
        log::info!("  queue_jumper: {} items", variables.queue_jumper.len());
        log::info!("  public_transport: {} items", variables.public_transport.len());
        log::info!("  seat_requester: {} items", variables.seat_requester.len());
        log::info!("  suspicious_behavior: {} items", variables.suspicious_behavior.len());
        log::info!("  survey_topic: {} items", variables.survey_topic.len());

        log::info!("Tier-specific variables:");
        log::info!("  colleague_descriptors: {} tiers", variables.colleague_descriptors.len());

        // Count and warn about empty variable lists
        let mut empty_vars = Vec::new();
        if variables.excuse_library.is_empty() { empty_vars.push("excuse_library"); }
        if variables.work_time.is_empty() { empty_vars.push("work_time"); }
        if variables.work_colleague.is_empty() { empty_vars.push("work_colleague"); }
        if variables.work_day.is_empty() { empty_vars.push("work_day"); }
        if variables.work_obligation.is_empty() { empty_vars.push("work_obligation"); }
        if variables.work_record.is_empty() { empty_vars.push("work_record"); }
        if variables.overtime_period.is_empty() { empty_vars.push("overtime_period"); }
        if variables.work_project.is_empty() { empty_vars.push("work_project"); }
        if variables.safety_violation.is_empty() { empty_vars.push("safety_violation"); }
        if variables.political_metric.is_empty() { empty_vars.push("political_metric"); }
        if variables.monitoring_target.is_empty() { empty_vars.push("monitoring_target"); }
        if variables.political_team_activity.is_empty() { empty_vars.push("political_team_activity"); }
        if variables.work_mistake.is_empty() { empty_vars.push("work_mistake"); }
        if variables.bribe_amount.is_empty() { empty_vars.push("bribe_amount"); }
        if variables.work_decision.is_empty() { empty_vars.push("work_decision"); }
        if variables.relationship_types.is_empty() { empty_vars.push("relationship_types"); }
        if variables.parent_type.is_empty() { empty_vars.push("parent_type"); }
        if variables.sibling_type.is_empty() { empty_vars.push("sibling_type"); }
        if variables.relative_type.is_empty() { empty_vars.push("relative_type"); }
        if variables.small_amount.is_empty() { empty_vars.push("small_amount"); }
        if variables.medium_amount.is_empty() { empty_vars.push("medium_amount"); }
        if variables.large_amount.is_empty() { empty_vars.push("large_amount"); }
        if variables.time_period.is_empty() { empty_vars.push("time_period"); }
        if variables.authority_figure.is_empty() { empty_vars.push("authority_figure"); }
        if variables.infraction.is_empty() { empty_vars.push("infraction"); }
        if variables.unpractical_subject.is_empty() { empty_vars.push("unpractical_subject"); }
        if variables.practical_subject.is_empty() { empty_vars.push("practical_subject"); }
        if variables.personal_topic.is_empty() { empty_vars.push("personal_topic"); }
        if variables.successful_relative.is_empty() { empty_vars.push("successful_relative"); }
        if variables.unsuitable_match.is_empty() { empty_vars.push("unsuitable_match"); }
        if variables.political_topic.is_empty() { empty_vars.push("political_topic"); }
        if variables.day_time.is_empty() { empty_vars.push("day_time"); }
        if variables.time_duration.is_empty() { empty_vars.push("time_duration"); }
        if variables.party_observer.is_empty() { empty_vars.push("party_observer"); }
        if variables.membership_level.is_empty() { empty_vars.push("membership_level"); }
        if variables.party_official.is_empty() { empty_vars.push("party_official"); }
        if variables.controversial_topic.is_empty() { empty_vars.push("controversial_topic"); }
        if variables.denouncement_target.is_empty() { empty_vars.push("denouncement_target"); }
        if variables.political_crime.is_empty() { empty_vars.push("political_crime"); }
        if variables.volunteer_activity.is_empty() { empty_vars.push("volunteer_activity"); }
        if variables.party_elite.is_empty() { empty_vars.push("party_elite"); }
        if variables.favor_request.is_empty() { empty_vars.push("favor_request"); }
        if variables.propaganda_campaign.is_empty() { empty_vars.push("propaganda_campaign"); }
        if variables.propaganda_activity.is_empty() { empty_vars.push("propaganda_activity"); }
        if variables.wait_time.is_empty() { empty_vars.push("wait_time"); }
        if variables.stranger_type.is_empty() { empty_vars.push("stranger_type"); }
        if variables.small_favor.is_empty() { empty_vars.push("small_favor"); }
        if variables.public_place.is_empty() { empty_vars.push("public_place"); }
        if variables.appointment_type.is_empty() { empty_vars.push("appointment_type"); }
        if variables.public_violation.is_empty() { empty_vars.push("public_violation"); }
        if variables.violation_perpetrator.is_empty() { empty_vars.push("violation_perpetrator"); }
        if variables.public_service.is_empty() { empty_vars.push("public_service"); }
        if variables.queue_jumper.is_empty() { empty_vars.push("queue_jumper"); }
        if variables.public_transport.is_empty() { empty_vars.push("public_transport"); }
        if variables.seat_requester.is_empty() { empty_vars.push("seat_requester"); }
        if variables.suspicious_behavior.is_empty() { empty_vars.push("suspicious_behavior"); }
        if variables.survey_topic.is_empty() { empty_vars.push("survey_topic"); }

        if !empty_vars.is_empty() {
            log::warn!("WARNING: {} empty variable lists: {:?}", empty_vars.len(), empty_vars);
        } else {
            log::info!("✓ All variable lists loaded successfully!");
        }

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
