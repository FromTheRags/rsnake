use crate::game_logic::high_score::HighScoreManager;
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    get_default_action_input, CellValue, FooterData, GenericMenu, RowData,
};
use ratatui::DefaultTerminal;

/// Sets up and runs the high-score table menu
pub fn setup_and_run_highs_table_parameters(terminal: &mut DefaultTerminal) {
    GenericMenu::new(
        load_highs_info_in_table(),
        &highs_get_headers(),
        highs_get_footer_data(),
        None,
    )
    .run(get_default_action_input(), terminal);
}

/// Loads highscore information into table rows for display
#[must_use]
fn load_highs_info_in_table() -> Vec<RowData> {
    let mut rows = vec![];

    if let Ok(manager) = HighScoreManager::new() {
        let high_scores = manager.get_top_scores();
        for (index, score) in high_scores.into_iter().enumerate() {
            rows.push(RowData::new(vec![
                CellValue::new(format!("#{}", index + 1)),
                CellValue::new(score.symbols),
                CellValue::new(format!("{}", score.score)),
                CellValue::new(score.speed),
                CellValue::new(format!("x{}", score.snake_growth_factor)),
                CellValue::new(score.date.format("%d/%m/%Y").to_string()),
                CellValue::new(score.version),
            ]));
        }
    }

    if rows.is_empty() {
        rows.push(RowData::new(vec![
            CellValue::new("No scores".to_string()),
            CellValue::new("Play the game!".to_string()),
            CellValue::new("🐍".to_string()),
            CellValue::new("-".to_string()),
            CellValue::new("-".to_string()),
            CellValue::new("-".to_string()),
            CellValue::new("-".to_string()),
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
        "🏁 Celerity ".to_string(),
        "📏 Growth".to_string(),
        "📅 Date".to_string(),
        "🐍 Version".to_string(),
    ]
}

/// Returns footer data for the high score table
#[must_use]
fn highs_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc/Tab".into(),
            text: "Return to home".into(),
            value: None,
        },
        FooterData {
            symbol: "↕".into(),
            text: "Scroll".into(),
            value: None,
        },
    ]
}
