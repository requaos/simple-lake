use super::library::{ChoiceArchetype, EventDomain, SituationTemplate};
use super::risk_calculator::{PlayerStats, calculate_risk};
use super::stat_calculator::{calculate_failure_stats, calculate_stats};
use super::text_assembly::{assemble_choice_text, assemble_description};
use crate::LotusApp;
use crate::game_data::{EventData, EventOption, EventOutcome};
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use std::collections::VecDeque;

/// Filter situations based on player state and context with detailed logging
fn filter_situations<'a>(
    situations: &'a [&'a SituationTemplate],
    player_tier: usize,
    life_stage: usize,
    recent_domains: &VecDeque<EventDomain>,
    encounter_history: &std::collections::HashSet<String>,
    allow_wildcard: bool,
) -> Vec<&'a SituationTemplate> {
    let total_situations = situations.len();
    log::debug!("Starting situation filtering with {} total situations", total_situations);
    log::debug!("  Player state: tier={}, life_stage={}", player_tier, life_stage);
    log::debug!("  Wildcard mode: {}", allow_wildcard);

    let last_two_domains: Vec<&EventDomain> = recent_domains.iter().take(2).collect();
    if !last_two_domains.is_empty() {
        log::debug!("  Recent domains (last 2): {:?}", last_two_domains.iter().map(|d| d.as_str()).collect::<Vec<_>>());
    }

    let mut tier_filtered = 0;
    let mut stage_filtered = 0;
    let mut encountered_filtered = 0;
    let mut domain_filtered = 0;

    let filtered: Vec<&'a SituationTemplate> = situations
        .iter()
        .copied()
        .filter(|s| {
            // Tier filter: player_tier ± 1
            let tier_ok = s.tier_min <= player_tier.saturating_add(1)
                && s.tier_max >= player_tier.saturating_sub(1);
            if !tier_ok {
                log::trace!("  FILTERED (tier): {} - tier_range=({}-{}), player_tier={}",
                    s.id, s.tier_min, s.tier_max, player_tier);
                tier_filtered += 1;
                return false;
            }

            // Life stage filter: current or previous stage
            let stage_ok = s.life_stage_min <= life_stage
                && s.life_stage_max >= life_stage.saturating_sub(1).max(1);
            if !stage_ok {
                log::trace!("  FILTERED (life_stage): {} - stage_range=({}-{}), player_stage={}",
                    s.id, s.life_stage_min, s.life_stage_max, life_stage);
                stage_filtered += 1;
                return false;
            }

            // Encounter history filter
            let not_encountered = !encounter_history.contains(&s.id);
            if !not_encountered {
                log::trace!("  FILTERED (already_encountered): {}", s.id);
                encountered_filtered += 1;
                return false;
            }

            // Recent domain filter (last 2 events)
            let domain_ok = allow_wildcard || !last_two_domains.contains(&&s.domain);
            if !domain_ok {
                log::trace!("  FILTERED (recent_domain): {} - domain={}", s.id, s.domain.as_str());
                domain_filtered += 1;
                return false;
            }

            log::trace!("  PASSED: {} (domain={}, tier={}-{}, stage={}-{})",
                s.id, s.domain.as_str(), s.tier_min, s.tier_max, s.life_stage_min, s.life_stage_max);
            true
        })
        .collect();

    log::info!("Situation filtering complete:");
    log::info!("  Total situations: {}", total_situations);
    log::info!("  Filtered by tier: {}", tier_filtered);
    log::info!("  Filtered by life_stage: {}", stage_filtered);
    log::info!("  Filtered by encounter_history: {}", encountered_filtered);
    log::info!("  Filtered by recent_domain: {}", domain_filtered);
    log::info!("  Remaining candidates: {}", filtered.len());

    filtered
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
pub fn generate_procedural_event(player_state: &LotusApp, rng: &mut impl Rng) -> Option<EventData> {
    log::info!("=== PROCEDURAL EVENT GENERATION ATTEMPT ===");

    let library = &player_state.situation_library;

    // 10% wildcard probability: ignore domain filter
    let allow_wildcard = rng.random_bool(0.1);
    if allow_wildcard {
        log::info!("WILDCARD mode activated - ignoring recent domain filter");
    }

    // Collect all situations from all domains
    let all_situations: Vec<&SituationTemplate> = library
        .by_domain
        .values()
        .flat_map(|situations| situations.iter())
        .collect();

    log::debug!("Total situations in library: {}", all_situations.len());

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
        log::warn!("PROCEDURAL GENERATION FAILED: No candidate situations after filtering");
        log::warn!("  Reason: All situations filtered out by tier/stage/history/domain criteria");
        log::warn!("  Will fall back to handcrafted events");
        return None;
    }

    // Weighted selection: prefer exact tier/stage matches
    let weights: Vec<f32> = candidates
        .iter()
        .map(|s| {
            let mut weight = 1.0;

            // Bonus for exact tier match
            if s.tier_min <= player_state.player_tier && s.tier_max >= player_state.player_tier {
                weight *= 2.0;
            }

            // Bonus for exact stage match
            if s.life_stage_min <= player_state.life_stage
                && s.life_stage_max >= player_state.life_stage
            {
                weight *= 2.0;
            }

            weight
        })
        .collect();

    // Weighted random selection
    let dist = WeightedIndex::new(&weights).ok()?;
    let selected_situation = candidates[dist.sample(rng)];

    log::info!("Selected situation: '{}' (domain={}, tier={}-{}, stage={}-{})",
        selected_situation.id,
        selected_situation.domain.as_str(),
        selected_situation.tier_min,
        selected_situation.tier_max,
        selected_situation.life_stage_min,
        selected_situation.life_stage_max
    );

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
    let total_choices = selected_situation.choices.len();
    log::debug!("Filtering {} choices by player requirements", total_choices);

    let available_choices: Vec<&ChoiceArchetype> = selected_situation
        .choices
        .iter()
        .filter(|c| {
            let meets_reqs = player_meets_requirements(player_state, &c.requirements);
            if !meets_reqs {
                log::debug!("  Choice '{}' filtered - requirements not met: {:?}",
                    c.archetype.as_str(), c.requirements);
            }
            meets_reqs
        })
        .collect();

    log::info!("Available choices: {}/{}", available_choices.len(), total_choices);

    // Must have at least one available choice
    if available_choices.is_empty() {
        log::warn!("PROCEDURAL GENERATION FAILED: No available choices");
        log::warn!("  Situation: '{}'", selected_situation.id);
        log::warn!("  Reason: All {} choices filtered by requirement checks", total_choices);
        log::warn!("  Player stats: career={}, family={}, network={}, party={}",
            player_state.career_level,
            player_state.guanxi_family,
            player_state.guanxi_network,
            player_state.guanxi_party
        );
        log::warn!("  Will fall back to handcrafted events");
        return None;
    }

    // Build EventOptions from available choices
    let options: Vec<EventOption> = available_choices
        .iter()
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
                if success_stats.scs_change > 0 {
                    "Things went well."
                } else {
                    "There were consequences."
                }
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

    log::info!("✓ PROCEDURAL EVENT GENERATION SUCCEEDED");
    log::info!("  Event: '{}' from domain '{}'", title, selected_situation.domain.as_str());
    log::info!("  Situation ID: '{}'", selected_situation.id);
    log::info!("  Options available: {}", options.len());

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
