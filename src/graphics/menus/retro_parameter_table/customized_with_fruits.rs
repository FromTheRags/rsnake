use crate::graphics::menus::retro_parameter_table::generic_logic::{
    get_default_action_input, CellValue, FooterData, GenericMenu, RowData,
};
use crate::graphics::sprites::fruit::FRUITS_SCORES_PROBABILITIES;
use ratatui::DefaultTerminal;

pub fn setup_and_run_fruits_table_parameters(terminal: &mut DefaultTerminal) {
    //a faire: extend ActionInput to have a callback for each action (like apply, move, change value)
    // Callback has a dyn &mut ApplyParameter and a &mut GenericMenu
    // Goal is to be allowed to reload the menu table with a preset and to move freely between between menu
    GenericMenu::new(
        load_fruits_info_in_table(),
        &fruits_get_headers(),
        fruits_get_footer_data(),
        get_default_action_input(),
        None,
    )
    .run(terminal);
}
/// Loads fruit information into table rows for display
/// Each row contains: Fruit emoji, Base Score, Spawn Chance, and Size Effect
#[must_use]
fn load_fruits_info_in_table() -> Vec<RowData> {
    let mut rows = vec![];

    for (fruit, score, probability, size_effect) in FRUITS_SCORES_PROBABILITIES {
        rows.push(RowData::new(vec![
            CellValue::new((*fruit).to_string()),
            CellValue::new(format!("{score}")),
            CellValue::new(format!("{probability}%")),
            CellValue::new(size_effect.to_string()),
        ]));
    }

    rows
}

/// Returns the header labels for fruits information table
#[must_use]
fn fruits_get_headers() -> Vec<String> {
    vec![
        "🍎 Fruit".to_string(),
        "🎯 Score x Speed Modifier".to_string(),
        "🎲 Chance".to_string(),
        "📏 Snake Size Effect".to_string(),
    ]
}

/// Should add an action to the Footer Data (like apply, move, change value)
#[must_use]
fn fruits_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc".into(),
            text: "Return to home".into(),
        },
        FooterData {
            symbol: "↕".into(),
            text: "Move".into(),
        },
    ]
}
