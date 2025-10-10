use crate::game_logic::game_options::{get_parameter_range, GameOptions, ONLY_FOR_CLI_PARAMETERS};
use crate::graphics::menus::generic_retro_parameter_logic::{CellValue, RowData};
use clap::CommandFactory;

#[must_use]
pub fn load_parameter_in_table(options: &GameOptions) -> Vec<RowData> {
    let cmd = GameOptions::command();
    let mut rows = vec![];
    let mut arg_value;
    for arg in cmd.get_arguments().filter(|arg| {
        !ONLY_FOR_CLI_PARAMETERS
            .iter()
            .any(|arg_pattern| arg.get_long().unwrap().contains(arg_pattern))
    }) {
        let mut values = vec![];
        //For booleans and enums, use clap functionalities to get possible values
        let pv = arg.get_possible_values();
        if pv.is_empty() {
            if let Some(range) = get_parameter_range(arg.get_long().unwrap()) {
                values.extend(range.map(|i| i.to_string()));
            } else {
                // If we are on Emoji String (the only no boolean, no range, no enum type there), if any other I would
                // have created a possible_value macro for each arguments: use the emoji vector to get them:
                values.extend(GameOptions::emojis_iterator());
            }
        } else {
            //For booleans and enums,
            values.extend(pv.into_iter().map(|v| v.get_name().to_string()));
        }
        // Set default value, from current value (default auto so if not set in CLI using default,
        // and serde default for missing serialize value for not-to-be-serialized value, for others...your fault to no provide them :p )
        let mut index = 0;
        //TOML crate prefers _ vs -
        if let Some(default_value) = options
            .to_structured_toml()
            .get(&arg.get_long().unwrap().replace('-', "_"))
        {
            let mut default_str = default_value.to_string();
            //TOML crate seems to love adding apostrophes and capitalize to string value
            if default_str.contains('"') {
                default_str = default_str.split('"').collect::<Vec<&str>>()[1]
                    .to_string()
                    .to_lowercase();
            }
            index = values.iter().position(|v| v == &default_str).unwrap_or(0);
        }
        //index = values.iter().position(|v| v == &default_str).unwrap_or(0);
        let arg_name = "--".to_string() + arg.get_long().unwrap();
        arg_value =
            crate::graphics::menus::generic_retro_parameter_logic::CellValue::new_with_options(
                arg_name, values, index,
            );

        rows.push(RowData::new(vec![
            arg_value,
            CellValue::new(arg.get_long().unwrap().to_string()),
            CellValue::new(
                arg.get_help()
                    .unwrap_or_else(|| {
                        panic!("Missing help for argument: {}", arg.get_long().unwrap())
                    })
                    .to_string(),
            ),
        ]));
    }
    rows
}
pub fn get_headers_parameters() -> Vec<String> {
    vec![
        "🎯 Value".to_string(),
        "📋 Parameter".to_string(),
        "📝 Description / super power".to_string(),
    ]
}
pub struct FooterData {
    pub symbol: String,
    pub text: String,
}
/// Should add an action to the Footer Data (like apply, move, change value)
#[must_use]
pub fn get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc".into(),
            text: "Apply".into(),
        },
        FooterData {
            symbol: "↕".into(),
            text: "Move".into(),
        },
        FooterData {
            symbol: "← →".into(),
            text: "Change value".into(),
        },
    ]
}
