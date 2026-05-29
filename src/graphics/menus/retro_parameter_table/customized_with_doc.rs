use crate::graphics::menus::retro_parameter_table::generic_logic::{
    CellValue, FooterData, GenericMenu, RowData, get_default_action_input,
};
use ratatui::DefaultTerminal;

/// Sets up and runs the doc menu using the generic retro table
pub fn setup_and_run_doc_table_parameters(terminal: &mut DefaultTerminal) {
    GenericMenu::new(
        load_doc_info_in_table(),
        &doc_get_headers(),
        doc_get_footer_data(),
        None,
    )
    .run(get_default_action_input(), terminal);
}

/// Loads doc / rules information into table rows for display
#[must_use]
fn load_doc_info_in_table() -> Vec<RowData> {
    let rows = vec![
        RowData::new(vec![
            CellValue::new("🎯 Goal".to_string()),
            CellValue::new("Eat fruits, grow, and score points.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("🍎 Fruits".to_string()),
            CellValue::new(
                "Different fruits give various scores and effects.\n\
             Some can even reduce your size (and score)."
                    .to_string(),
            ),
        ]),
        RowData::new(vec![
            CellValue::new("🏁 Speed".to_string()),
            CellValue::new(
                "Game speed changes difficulty and score multipliers.\n\
             See the Speed menu for more details."
                    .to_string(),
            ),
        ]),
        RowData::new(vec![
            CellValue::new("🎮 Movement".to_string()),
            CellValue::new("Use ← ↕ → to control the snake.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("💥 Collisions".to_string()),
            CellValue::new("Avoid hitting yourself or your own tail.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("🌍 Walls".to_string()),
            CellValue::new(
                "The arena wraps around: going out on one side\n\
             makes you appear on the opposite side."
                    .to_string(),
            ),
        ]),
        RowData::new(vec![
            CellValue::new("⏸️ Pause".to_string()),
            CellValue::new("Press P or Space to pause / resume.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("🔁 New game".to_string()),
            CellValue::new("Press R to start a new game.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("📋 Main menu".to_string()),
            CellValue::new("Press M to return to the main menu.".to_string()),
        ]),
        RowData::new(vec![
            CellValue::new("🚪 Quit".to_string()),
            CellValue::new("Press Q to quit at any time.".to_string()),
        ]),
    ];
    rows
}

/// Returns the header labels for the doc information table
#[must_use]
fn doc_get_headers() -> Vec<String> {
    vec!["📖 Topic".to_string(), "ℹ️ Details".to_string()]
}

/// Returns footer data for the doc table
#[must_use]
fn doc_get_footer_data() -> Vec<FooterData> {
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
