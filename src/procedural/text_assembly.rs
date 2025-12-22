use super::library::{NarrativeFragments, VariableLibraries};
use rand::prelude::*;

pub fn assemble_description(
    fragments: &NarrativeFragments,
    variables: &VariableLibraries,
    player_tier: usize,
    rng: &mut impl Rng,
) -> String {
    let opening = fragments
        .openings
        .choose(rng)
        .expect("No opening fragments")
        .clone();
    let conflict = fragments
        .conflicts
        .choose(rng)
        .expect("No conflict fragments")
        .clone();
    let stakes = fragments
        .stakes
        .choose(rng)
        .expect("No stakes fragments")
        .clone();

    let mut text = format!("{} {} {}", opening, conflict, stakes);

    // Variable substitution
    text = substitute_variables(text, variables, player_tier, rng);

    text
}

pub fn assemble_choice_text(text_fragments: &[String], rng: &mut impl Rng) -> String {
    text_fragments
        .choose(rng)
        .expect("No choice text fragments")
        .clone()
}

fn substitute_variables(
    mut text: String,
    variables: &VariableLibraries,
    player_tier: usize,
    rng: &mut impl Rng,
) -> String {
    // Helper macro to substitute a variable placeholder with a random choice from a list
    macro_rules! substitute {
        ($placeholder:expr, $list:expr) => {
            if !$list.is_empty() && text.contains($placeholder) {
                if let Some(value) = $list.choose(rng) {
                    text = text.replace($placeholder, value);
                }
            }
        };
    }

    // Substitute {colleague_descriptor} (tier-specific)
    if text.contains("{colleague_descriptor}") {
        let descriptors = variables
            .colleague_descriptors
            .get(&player_tier.to_string())
            .or_else(|| variables.colleague_descriptors.get("2"))
            .expect("No colleague descriptors");
        if let Some(descriptor) = descriptors.choose(rng) {
            text = text.replace("{colleague_descriptor}", descriptor);
        }
    }

    // Work variables
    substitute!("{excuse}", &variables.excuse_library);
    substitute!("{work_time}", &variables.work_time);
    substitute!("{work_colleague}", &variables.work_colleague);
    substitute!("{work_day}", &variables.work_day);
    substitute!("{work_obligation}", &variables.work_obligation);
    substitute!("{work_record}", &variables.work_record);
    substitute!("{overtime_period}", &variables.overtime_period);
    substitute!("{work_project}", &variables.work_project);
    substitute!("{safety_violation}", &variables.safety_violation);
    substitute!("{political_metric}", &variables.political_metric);
    substitute!("{monitoring_target}", &variables.monitoring_target);
    substitute!("{political_team_activity}", &variables.political_team_activity);
    substitute!("{work_mistake}", &variables.work_mistake);
    substitute!("{bribe_amount}", &variables.bribe_amount);
    substitute!("{work_decision}", &variables.work_decision);

    // Family variables
    substitute!("{relationship_type}", &variables.relationship_types);
    substitute!("{parent_type}", &variables.parent_type);
    substitute!("{sibling_type}", &variables.sibling_type);
    substitute!("{relative_type}", &variables.relative_type);
    substitute!("{small_amount}", &variables.small_amount);
    substitute!("{medium_amount}", &variables.medium_amount);
    substitute!("{large_amount}", &variables.large_amount);
    substitute!("{time_period}", &variables.time_period);
    substitute!("{authority_figure}", &variables.authority_figure);
    substitute!("{infraction}", &variables.infraction);
    substitute!("{unpractical_subject}", &variables.unpractical_subject);
    substitute!("{practical_subject}", &variables.practical_subject);
    substitute!("{personal_topic}", &variables.personal_topic);
    substitute!("{successful_relative}", &variables.successful_relative);
    substitute!("{unsuitable_match}", &variables.unsuitable_match);

    // Party variables
    substitute!("{political_topic}", &variables.political_topic);
    substitute!("{day_time}", &variables.day_time);
    substitute!("{time_duration}", &variables.time_duration);
    substitute!("{party_observer}", &variables.party_observer);
    substitute!("{membership_level}", &variables.membership_level);
    substitute!("{party_official}", &variables.party_official);
    substitute!("{controversial_topic}", &variables.controversial_topic);
    substitute!("{denouncement_target}", &variables.denouncement_target);
    substitute!("{political_crime}", &variables.political_crime);
    substitute!("{volunteer_activity}", &variables.volunteer_activity);
    substitute!("{party_elite}", &variables.party_elite);
    substitute!("{favor_request}", &variables.favor_request);
    substitute!("{propaganda_campaign}", &variables.propaganda_campaign);
    substitute!("{propaganda_activity}", &variables.propaganda_activity);

    // Public variables
    substitute!("{stranger_type}", &variables.stranger_type);
    substitute!("{small_favor}", &variables.small_favor);
    substitute!("{public_place}", &variables.public_place);
    substitute!("{appointment_type}", &variables.appointment_type);
    substitute!("{public_violation}", &variables.public_violation);
    substitute!("{violation_perpetrator}", &variables.violation_perpetrator);
    substitute!("{public_service}", &variables.public_service);
    substitute!("{queue_jumper}", &variables.queue_jumper);
    substitute!("{public_transport}", &variables.public_transport);
    substitute!("{seat_requester}", &variables.seat_requester);
    substitute!("{suspicious_behavior}", &variables.suspicious_behavior);
    substitute!("{survey_topic}", &variables.survey_topic);

    text
}
