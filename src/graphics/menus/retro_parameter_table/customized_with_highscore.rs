use crate::graphics::menus::retro_parameter_table::generic_logic::{
    get_default_action_input, CellValue, FooterData, GenericMenu, RowData,
};
use ratatui::DefaultTerminal;

/// Sets up and runs the high score table menu
pub fn setup_and_run_highs_table_parameters(terminal: &mut DefaultTerminal) {
    GenericMenu::new(
        load_highs_info_in_table(),
        &highs_get_headers(),
        highs_get_footer_data(),
    )
    .run(get_default_action_input(), terminal);
}

/// Loads high score information into table rows for display
#[must_use]
fn load_highs_info_in_table() -> Vec<RowData> {
    let mut rows = vec![];

    // TODO: Replace with actual high score loading logic from game state or file
    let high_scores = vec![
        (
            1,
            "YET TO IMPLEMENT :p ",
            5000,
            "Fast",
            "09/12/2025",
            "0.1.2",
        ),
        (
            2,
            "CONTRIBUTE TO THIS PROJECT",
            3500,
            "Normal",
            "01/01/2025",
            "0.1.2",
        ),
        (
            3,
            "TO HAVE THIS FEATURE",
            1200,
            "Slow",
            "31/12/2024",
            "0.1.1",
        ),
    ];

    for (rank, symbols, score, speed, date, version) in high_scores {
        rows.push(RowData::new(vec![
            CellValue::new(format!("#{rank}")),
            CellValue::new(symbols.to_string()),
            CellValue::new(format!("{score}")),
            CellValue::new(speed.to_string()),
            CellValue::new(date.to_string()),
            CellValue::new(version.to_string()),
        ]));
    }

    rows
}

/// Returns the header labels for the high score information table
#[must_use]
fn highs_get_headers() -> Vec<String> {
    vec![
        "🏆 Rank".to_string(),
        "👤 Head & Body".to_string(),
        "🎯 Score".to_string(),
        "🏁 Speed".to_string(),
        "📅 Date".to_string(),
        "🏷️ Version".to_string(),
    ]
}

/// Returns footer data for the high score table
#[must_use]
fn highs_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc".into(),
            text: "Return to home".into(),
        },
        FooterData {
            symbol: "↕".into(),
            text: "Scroll".into(),
        },
    ]
}
