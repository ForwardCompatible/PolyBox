// src/actions/ordering.rs
// Action Execution Ordering
//
// Purpose: Sorts parsed actions into correct execution order according to SPEC.md §1.2
// Responsibilities:
// - Order actions by execution type: write first, then read, then event
// - Preserve original appearance order within each execution type category
// - Place actions with unknown execution type last
//
// Dependencies:
// - ParsedAction from parser module
// - ActionRegistryEntry for execution type lookup
//
// Created: 2026-04-20

use super::{ParsedAction, ActionRegistryEntry};

/// Sorts actions into the correct execution order
/// 
/// Execution priority order:
/// 1. Write actions (highest priority)
/// 2. Read actions
/// 3. Event actions
/// 4. Unknown execution type (lowest priority)
/// 
/// Original order is preserved within each priority group.
pub fn order_actions_for_execution(actions: &[ParsedAction], registry: &[ActionRegistryEntry]) -> Vec<ParsedAction> {
    // Assign priority values for stable sorting
    fn get_execution_priority(tag_name: &str, registry: &[ActionRegistryEntry]) -> u8 {
        registry.iter()
            .find(|entry| entry.tag_name == tag_name)
            .map(|entry| match entry.execution_type.as_str() {
                "write" => 0,
                "read" => 1,
                "event" => 2,
                _ => 3,
            })
            .unwrap_or(3)
    }

    // Create indexed list to preserve original order
    let mut indexed_actions: Vec<(usize, ParsedAction)> = actions
        .iter()
        .cloned()
        .enumerate()
        .collect();

    // Stable sort by execution priority first, then original index
    indexed_actions.sort_by_key(|(index, action)| {
        (get_execution_priority(&action.tag_name, registry), *index)
    });

    // Extract sorted actions
    indexed_actions
        .into_iter()
        .map(|(_, action)| action)
        .collect()
}
