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
    // Substitute {colleague_descriptor}
    if text.contains("{colleague_descriptor}") {
        let descriptors = variables
            .colleague_descriptors
            .get(&player_tier.to_string())
            .or_else(|| variables.colleague_descriptors.get("2"))
            .expect("No colleague descriptors");
        let descriptor = descriptors.choose(rng).expect("Empty descriptor list");
        text = text.replace("{colleague_descriptor}", descriptor);
    }

    // Substitute {excuse}
    if text.contains("{excuse}") {
        let excuse = variables
            .excuse_library
            .choose(rng)
            .expect("Empty excuse library");
        text = text.replace("{excuse}", excuse);
    }

    // Substitute {relationship_type}
    if text.contains("{relationship_type}") {
        let relationship = variables
            .relationship_types
            .choose(rng)
            .expect("Empty relationship types");
        text = text.replace("{relationship_type}", relationship);
    }

    text
}
