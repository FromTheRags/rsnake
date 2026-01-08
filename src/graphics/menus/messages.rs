use crate::game_logic::state::GameOverMenu;
use crate::graphics::menus::main_menu::Button;
use crate::graphics::menus::utils_layout::{
    frame_vertically_centered_rect, render_full_centered_paragraph,
};
use ratatui::style::Color;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

// For ASCII art use AI or a generator as:
// http://patorjk.com/software/taag/#p=display&f=ANSI%20Shadow&t=Pause
// For text display: https://ratatui.rs/recipes/render/display-text/
//Or (but use a dependency): https://crates.io/crates/tui-big-text
pub const SNAKE_LOGO: &str = "\
███████╗ ███╗   ██╗  █████╗  ██╗  ██╗ ███████╗
██╔════╝ ████╗  ██║ ██╔══██╗ ██║ ██╔╝ ██╔════╝
███████╗ ██╔██╗ ██║ ███████║ █████╔╝  █████╗  
╚════██║ ██║╚██╗██║ ██╔══██║ ██╔═██╗  ██╔══╝  
███████║ ██║ ╚████║ ██║  ██║ ██║  ██╗ ███████╗
╚══════╝ ╚═╝  ╚═══╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝";

// Define the control table as a constant
pub const CONTROLS_TABLE: &str = "\
+--------+------+------+-------+-----+-------+
|Controls| ←↕→  |  Q   | P / ⎵ |  M  |   R   |
+--------+------+------+-------+-----+-------+
|Effects | Move | Quit | Pause | Menu| Start |
+--------+------+------+-------+-----+-------+";
const GAME_OVER_TEXT: &str = "\n\
 ██████╗  █████╗ ███╗   ███╗███████╗       ██████╗ ██╗   ██╗███████╗██████╗ \n\
██╔═══   ██╔══██╗████╗ ████║██╔════╝      ██╔═══██╗██║   ██║██╔════╝██╔══██╗\n\
██║ ████║███████║██╔████╔██║█████╗        ██║   ██║██║   ██║█████╗  ██████╔╝\n\
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝        ██║   ██║██║   ██║██╔══╝  ██╔══██╗\n\
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗      ╚██████╔╝╚██████╔╝███████╗██║  ██║\n\
 ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝       ╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝";

const PAUSE_TEXT: &str = "\n\
██████╗  █████╗ ██╗   ██╗███████╗███████╗\n\
██╔══██╗██╔══██╗██║   ██║██╔════╝██╔════╝\n\
██████╔╝███████║██║   ██║███████╗█████╗  \n\
██╔═══╝ ██╔══██║██║   ██║╚════██║██╔══╝  \n\
██║     ██║  ██║╚██████╔╝███████║███████╗\n\
╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝";
//goodbye
const MENU_TEXT: &str = "\n\
███╗   ███╗███████╗███╗   ██╗██╗   ██╗
████╗ ████║██╔════╝████╗  ██║██║   ██║
██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║
██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║
██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝
╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ";

const FAREWELL_TEXT: &str = "\n\
██████╗ ██╗   ██╗███████╗    ██████╗ ██╗   ██╗███████╗
██╔══██╗╚██╗ ██╔╝██╔════╝    ██╔══██╗╚██╗ ██╔╝██╔════╝
██████╔╝ ╚████╔╝ █████╗█████╗██████╔╝ ╚████╔╝ █████╗  
██╔══██╗  ╚██╔╝  ██╔══╝╚════╝██╔══██╗  ╚██╔╝  ██╔══╝  
██████╔╝   ██║   ███████╗    ██████╔╝   ██║   ███████╗
╚═════╝    ╚═╝   ╚══════╝    ╚═════╝    ╚═╝   ╚══════╝";

const RESTART_TEXT: &str = "\n\
██████╗ ███████╗███████╗████████╗ █████╗ ██████╗ ████████╗
██╔══██╗██╔════╝██╔════╝╚══██╔══╝██╔══██╗██╔══██╗╚══██╔══╝
██████╔╝█████╗  ███████╗   ██║   ███████║██████╔╝   ██║   
██╔══██╗██╔══╝  ╚════██║   ██║   ██╔══██║██╔══██╗   ██║   
██║  ██║███████╗███████║   ██║   ██║  ██║██║  ██║   ██║   
╚═╝  ╚═╝╚══════╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ";

pub fn pause_paragraph(frame: &mut Frame) {
    render_full_centered_paragraph(frame, PAUSE_TEXT, None);
}
pub fn byebye_paragraph(frame: &mut Frame) {
    render_full_centered_paragraph(frame, FAREWELL_TEXT, Some(Color::DarkGray));
}
pub fn restart_paragraph(frame: &mut Frame) {
    render_full_centered_paragraph(frame, RESTART_TEXT, Some(Color::LightYellow));
}
pub fn menu_paragraph(frame: &mut Frame) {
    //Rgb Purple badly managed by old terminals Color::Rgb(128, 0, 128)
    render_full_centered_paragraph(frame, MENU_TEXT, Some(Color::Gray));
}
pub fn game_over_paragraph(
    frame: &mut Frame,
    selection: &GameOverMenu,
    score: u32,
    rank: Option<usize>,
) {
    let mut lines = vec![];
    //Get the GAME OVER text in red
    for logo_line in GAME_OVER_TEXT.lines() {
        lines.push(Line::from(logo_line).style(ratatui::style::Style::default().fg(Color::Red)));
    }

    //Display the score text
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::raw("Score: "),
        Span::styled(
            format!("{score}"),
            ratatui::style::Style::default().fg(Color::Yellow),
        ),
    ]));
    // if among top 10, display the ranking
    if let Some(r) = rank {
        lines.push(Line::from(vec![
            Span::raw("Ranking: "),
            Span::styled(
                format!("#{r}"),
                ratatui::style::Style::default().fg(Color::Green),
            ),
            Span::raw(" among top 10!"),
        ]));
    }
    lines.push(Line::default());

    // Display the option buttons to restart/Menu/Quit for the player
    let mut restart_button = Button::new("Restart 🎮");
    let mut menu_button = Button::new("Menu 🗺");
    let mut quit_button = Button::new("Quit ");

    restart_button.selected(selection == &GameOverMenu::Restart);
    menu_button.selected(selection == &GameOverMenu::Menu);
    quit_button.selected(selection == &GameOverMenu::Quit);

    let mut combined_spans: Vec<Span> = restart_button.to_spans();
    combined_spans.extend(menu_button.to_spans());
    combined_spans.extend(quit_button.to_spans());
    lines.push(Line::from(combined_spans));

    // Render all the previous elements
    let nb_lines = lines.len();
    frame.render_widget(
        Paragraph::new(Text::from(lines)).centered(),
        frame_vertically_centered_rect(frame.area(), nb_lines),
    );
    //Render the frame all around the edges
    //NB: no red everywhere on block as with "//.style(ratatui::style::Style::default().fg(Color::Red)),"
    //Because otherwise the button's color of the game over screen will be lost
    frame.render_widget(ratatui::widgets::Block::default(), frame.area());
}
