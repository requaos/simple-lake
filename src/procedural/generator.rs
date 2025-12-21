use super::library::{EventDomain, SituationTemplate, ChoiceArchetype};
use super::text_assembly::{assemble_description, assemble_choice_text};
use super::stat_calculator::{calculate_stats, calculate_failure_stats};
use super::risk_calculator::{calculate_risk, PlayerStats};
use crate::game_data::{EventData, EventOption, EventOutcome};
use crate::LotusApp;
use rand::prelude::*;
use rand::distr::weighted::WeightedIndex;
use std::collections::VecDeque;

/// Filter situations based on player state and context
fn filter_situations<'a>(
    situations: &'a [&'a SituationTemplate],
    player_tier: usize,
    life_stage: usize,
    recent_domains: &VecDeque<EventDomain>,
    encounter_history: &std::collections::HashSet<String>,
    allow_wildcard: bool,
) -> Vec<&'a SituationTemplate> {
    situations.iter()
        .copied()
        .filter(|s| {
            // Tier filter: player_tier ± 1
            let tier_ok = s.tier_min <= player_tier.saturating_add(1)
                && s.tier_max >= player_tier.saturating_sub(1);

            // Life stage filter: current or previous stage
            let stage_ok = s.life_stage_min <= life_stage
                && s.life_stage_max >= life_stage.saturating_sub(1).max(1);

            // Encounter history filter
            let not_encountered = !encounter_history.contains(&s.id);

            // Recent domain filter (last 2 events)
            let last_two_domains: Vec<&EventDomain> = recent_domains.iter().take(2).collect();
            let domain_ok = allow_wildcard || !last_two_domains.contains(&&s.domain);

            tier_ok && stage_ok && not_encountered && domain_ok
        })
        .collect()
}

/// Check if player meets requirements for a choice
fn player_meets_requirements(
    player_state: &LotusApp,
    requirements: &std::collections::HashMap<String, u32>,
) -> bool {
    for (key, &required_value) in requirements {
        let player_value = match key.as_str() {
            "guanxi_family" => player_state.guanxi_family,
            "guanxi_network" => player_state.guanxi_network,
            "guanxi_party" => player_state.guanxi_party,
            "career_level" => player_state.career_level,
            _ => 0,
        };

        if player_value < required_value {
            return false;
        }
    }
    true
}

/// Generate a procedural event based on player state
pub fn generate_procedural_event(
    player_state: &LotusApp,
    rng: &mut impl Rng,
) -> Option<EventData> {
    let library = &player_state.situation_library;

    // 10% wildcard probability: ignore domain filter
    let allow_wildcard = rng.random_bool(0.1);

    // Collect all situations from all domains
    let all_situations: Vec<&SituationTemplate> = library.by_domain
        .values()
        .flat_map(|situations| situations.iter())
        .collect();

    // Filter situations based on player state and context
    let candidates = filter_situations(
        &all_situations,
        player_state.player_tier,
        player_state.life_stage,
        &player_state.recent_event_domains,
        &player_state.encounter_history,
        allow_wildcard,
    );

    if candidates.is_empty() {
        return None;
    }

    // Weighted selection: prefer exact tier/stage matches
    let weights: Vec<f32> = candidates.iter().map(|s| {
        let mut weight = 1.0;

        // Bonus for exact tier match
        if s.tier_min <= player_state.player_tier && s.tier_max >= player_state.player_tier {
            weight *= 2.0;
        }

        // Bonus for exact stage match
        if s.life_stage_min <= player_state.life_stage && s.life_stage_max >= player_state.life_stage {
            weight *= 2.0;
        }

        weight
    }).collect();

    // Weighted random selection
    let dist = WeightedIndex::new(&weights).ok()?;
    let selected_situation = candidates[dist.sample(rng)];

    // Generate event description
    let description = assemble_description(
        &selected_situation.fragments,
        &library.variables,
        player_state.player_tier,
        rng,
    );

    // Generate title from domain and severity
    let title = format!(
        "{} - {} Severity",
        selected_situation.domain.as_str(),
        match selected_situation.severity {
            super::library::Severity::Low => "Low",
            super::library::Severity::Medium => "Medium",
            super::library::Severity::High => "High",
        }
    );

    // Assemble choices - filter by requirements
    let available_choices: Vec<&ChoiceArchetype> = selected_situation.choices.iter()
        .filter(|c| player_meets_requirements(player_state, &c.requirements))
        .collect();

    // Must have at least one available choice
    if available_choices.is_empty() {
        return None;
    }

    // Build EventOptions from available choices
    let options: Vec<EventOption> = available_choices.iter()
        .map(|choice| {
            // Generate choice text
            let text = assemble_choice_text(&choice.text_fragments, rng);

            // Calculate context-driven stats
            let success_stats = calculate_stats(
                &choice.base_stats,
                player_state.player_tier,
                selected_situation.severity,
                rng,
            );

            // Calculate failure stats (inverted/amplified)
            let failure_stats = calculate_failure_stats(&success_stats);

            // Calculate risk
            let player_stats = PlayerStats {
                guanxi_family: player_state.guanxi_family,
                guanxi_network: player_state.guanxi_network,
                guanxi_party: player_state.guanxi_party,
                career_level: player_state.career_level,
            };

            let risk_chance = calculate_risk(
                selected_situation.base_risk,
                choice.risk_modifier,
                &choice.requirements,
                &player_stats,
            );

            // Generate result text
            let success_result = format!(
                "You chose to {}. {}",
                choice.archetype.as_str(),
                if success_stats.scs_change > 0 { "Things went well." } else { "There were consequences." }
            );

            let failure_result = format!(
                "You chose to {}, but it backfired. Things didn't go as planned.",
                choice.archetype.as_str()
            );

            EventOption {
                text,
                requirements: choice.requirements.clone(),
                risk_chance,
                success_outcome: EventOutcome {
                    scs_change: success_stats.scs_change,
                    finance_change: success_stats.finance_change,
                    career_level_change: success_stats.career_level_change,
                    guanxi_family_change: success_stats.guanxi_family_change,
                    guanxi_network_change: success_stats.guanxi_network_change,
                    guanxi_party_change: success_stats.guanxi_party_change,
                },
                success_result,
                failure_outcome: Some(EventOutcome {
                    scs_change: failure_stats.scs_change,
                    finance_change: failure_stats.finance_change,
                    career_level_change: failure_stats.career_level_change,
                    guanxi_family_change: failure_stats.guanxi_family_change,
                    guanxi_network_change: failure_stats.guanxi_network_change,
                    guanxi_party_change: failure_stats.guanxi_party_change,
                }),
                failure_result,
            }
        })
        .collect();

    Some(EventData {
        title,
        description,
        options,
        min_tier: selected_situation.tier_min,
        max_tier: selected_situation.tier_max,
        is_generic: false,
        life_stage: player_state.life_stage,
        procedural_id: Some(selected_situation.id.clone()),
        procedural_domain: Some(selected_situation.domain.as_str().to_string()),
    })
}
