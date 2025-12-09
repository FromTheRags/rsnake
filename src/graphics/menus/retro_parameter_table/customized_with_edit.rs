use crate::game_logic::game_options::{get_parameter_range, GameOptions, ONLY_FOR_CLI_PARAMETERS};
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    get_default_action_input, ActionInputs, CellValue, FooterData, GenericMenu, RowData,
    TableParameterAction,
};
use clap::CommandFactory;
use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

pub fn setup_and_run_cli_table_parameters(
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) {
    let data: Vec<RowData> = load_parameter_cli_in_table(options);
    let mut actions = get_default_action_input();
    actions.push(ActionInputs {
        key: vec![KeyCode::Char('x'), KeyCode::Char('X')],
        action: vec![
            TableParameterAction::Apply(options),
            TableParameterAction::Quit,
        ],
    });
    actions.push(ActionInputs {
        key: vec![KeyCode::Char('1')],
        //Lazy init of the game options from preset 1
        action: vec![TableParameterAction::LoadPreset(1, || {
            load_parameter_cli_in_table(&mut GameOptions::load_from_toml_preset(1).unwrap())
        })],
    });
    // Call Parameters screen and input management with the game options to modify
    GenericMenu::new(
        data,
        &parameters_cli_get_headers(),
        parameters_cli_get_footer_data(),
    )
    .run(actions, terminal);
}

#[must_use]
fn load_parameter_cli_in_table(options: &mut GameOptions) -> Vec<RowData> {
    let cmd = GameOptions::command();
    let mut rows = vec![];
    let mut arg_value;
    //We get all the possible arguments for the game (not the ones for CLI tweaks)
    //to have the same arguments tweakable with the in-game menu as the cli
    for arg in cmd.get_arguments().filter(|arg| {
        !ONLY_FOR_CLI_PARAMETERS
            .iter()
            .any(|arg_pattern| arg.get_long().unwrap().contains(arg_pattern))
    }) {
        //We want all the possible values for the argument to have a selectable list of them
        let mut all_value_for_this_arg = vec![];
        //For booleans and enums, use clap functionalities to get possible values
        // get_possible_values() only works for boolean or enum
        let pv_bool_enum = arg.get_possible_values();
        if pv_bool_enum.is_empty() {
            if let Some(range) = get_parameter_range(arg.get_long().unwrap()) {
                all_value_for_this_arg.extend(range.map(|i| i.to_string()));
            } else {
                // If we are on Emoji String (the only no boolean, no range, no enum type there),
                // if any other I have created a possible_value macro for each argument:
                // use the emoji vector to get them:
                all_value_for_this_arg.extend(GameOptions::emojis_iterator());
                //Check if the user provides an existing emoji or a brand new one

                add_unique_symbol(&mut all_value_for_this_arg, options.head_symbol.clone());
                add_unique_symbol(&mut all_value_for_this_arg, options.body_symbol.clone());
            }
        } else {
            //For booleans and enums,
            all_value_for_this_arg
                .extend(pv_bool_enum.into_iter().map(|v| v.get_name().to_string()));
        }
        // Set default value, from current value (default auto so if not set in CLI using default,
        // and serde default for missing serialize value for not-to-be-serialized value,
        // for others...your fault to no provides them :p)
        let mut index = 0;
        //TOML crate prefers _ vs. -
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
            index = all_value_for_this_arg
                .iter()
                .position(|v| v == &default_str)
                .unwrap_or(0);
        }
        //index = values.iter().position(|v| v == &default_str).unwrap_or(0);
        let arg_name = "--".to_string() + arg.get_long().unwrap();
        arg_value = CellValue::new_with_options(arg_name, all_value_for_this_arg, index);

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
#[must_use]
fn parameters_cli_get_headers() -> Vec<String> {
    vec![
        "🎯 Value".to_string(),
        "📋 Parameter".to_string(),
        "📝 Description / super power".to_string(),
    ]
}
/// Should add an action to the Footer Data (like apply, move, change value)
#[must_use]
fn parameters_cli_get_footer_data() -> Vec<FooterData> {
    vec![
        FooterData {
            symbol: "Esc".into(),
            text: "Quit".into(),
        },
        FooterData {
            symbol: "x".into(),
            text: "Quit & Apply".into(),
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

fn add_unique_symbol(collection: &mut Vec<String>, symbol: String) {
    if !collection.contains(&symbol) {
        collection.push(symbol);
    }
}
