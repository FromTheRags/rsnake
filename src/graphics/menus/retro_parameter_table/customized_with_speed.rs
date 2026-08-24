use crate::controls::speed::Speed;
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    CellValue, FooterData, GenericMenu, RowData, get_default_action_input,
};
use clap::ValueEnum;
use ratatui::DefaultTerminal;

/// Sets up and runs the speed table parameters menu
pub fn setup_and_run_speed_table_parameters(terminal: &mut DefaultTerminal) {
    GenericMenu::new(
        load_speed_info_in_table(),
        &speed_get_headers(),
        speed_get_footer_data(),
        None,
    )
    .run(get_default_action_input(), terminal);
}

/// Loads speed information into table rows for display
/// Each row contains: Speed name, delay value, score modifier, and symbol
#[must_use]
fn load_speed_info_in_table() -> Vec<RowData> {
    let mut rows = vec![];

    // For each speed variant, extract its configuration and add to rows
    for speed in Speed::value_variants() {
        let config = speed.config();
        rows.push(RowData::new(vec![
            CellValue::new(config.name.to_string()),
            CellValue::new(format!("{}ms", config.ms_value)),
            CellValue::new(format!("x{}", config.score_modifier)),
            CellValue::new(config.symbol.to_string()),
        ]));
    }

    rows
}

/// Returns the header labels for the speed information table
#[must_use]
fn speed_get_headers() -> Vec<String> {
    vec![
        "🏁 Speed Level".to_string(),
        "⏱️ Delay Value".to_string(),
        "🎯 Score Modifier".to_string(),
        "🔣 Symbol".to_string(),
    ]
}

/// Returns footer data for the speed parameters table
#[must_use]
fn speed_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc/Tab".into(),
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
