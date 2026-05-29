use crate::game_logic::game_options::GameOptions;
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    get_default_action_input, CellValue, FooterData, GenericMenu, RowData,
};
use crate::graphics::sprites::fruit::{Fruit, FRUITS_SCORES_PROBABILITIES};
use ratatui::DefaultTerminal;
use std::time::Duration;

pub fn setup_and_run_fruits_table_parameters(
    terminal: &mut DefaultTerminal,
    options: &GameOptions,
) {
    //later: extend ActionInput to have a callback for each action (like apply, move, change value)
    // Callback has a dyn &mut ApplyParameter and a &mut GenericMenu
    // Goal is to be allowed to reload the menu table with a preset and to move freely between menus.
    GenericMenu::new(
        load_fruits_info_in_table(options),
        &fruits_get_headers(),
        fruits_get_footer_data(),
        None,
    )
    .run(get_default_action_input(), terminal);
}

/// Loads fruit information into table rows for display.
/// Each row contains: Fruit emoji, Base Score, Spawn Chance, Size Effect, and Computed duration.
#[must_use]
fn load_fruits_info_in_table(options: &GameOptions) -> Vec<RowData> {
    let mut rows = vec![];
    let base_duration = Duration::from_secs(u64::from(options.fruit_duration_seconds));

    for (fruit, score, probability, size_effect) in FRUITS_SCORES_PROBABILITIES {
        let multiplier = Fruit::duration_multiplier(*size_effect);
        let computed_duration = Fruit::duration_from_base(base_duration, *size_effect);
        rows.push(RowData::new(vec![
            CellValue::new((*fruit).to_string()),
            CellValue::new(format!("{score}")),
            CellValue::new(format!("{probability}%")),
            CellValue::new(size_effect.to_string()),
            CellValue::new(format!(
                "{:.1}s ({:.0} x {multiplier:.1})",
                computed_duration.as_secs_f32(),
                base_duration.as_secs_f32()
            )),
        ]));
    }

    rows
}

/// Returns the header labels for fruits information table.
#[must_use]
fn fruits_get_headers() -> Vec<String> {
    vec![
        "🍎 Fruit".to_string(),
        "🎯 Score x Speed Modifier".to_string(),
        "🎲 Chance".to_string(),
        "📏 Snake Size Effect".to_string(),
        "⏱ Computed".to_string(),
    ]
}

/// Should add an action to the Footer Data (like apply, move, change value).
#[must_use]
fn fruits_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc".into(),
            text: "Return to home".into(),
            value: None,
        },
        FooterData {
            symbol: "↕".into(),
            text: "Move".into(),
            value: None,
        },
    ]
}
