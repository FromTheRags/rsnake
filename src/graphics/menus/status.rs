use crate::graphics::menus::utils_layout::render_full_centered_paragraph;
use ratatui::Frame;
use ratatui::style::Color;

// For ASCII art use AI or a generator as:
// http://patorjk.com/software/taag/#p=display&f=ANSI%20Shadow&t=Pause
// For text display: https://ratatui.rs/recipes/render/display-text/
const GAME_OVER_TEXT: &str = "\n\
 ██████╗  █████╗ ███╗   ███╗███████╗       ██████╗ ██╗   ██╗███████╗██████╗ \n\
██╔═══   ██╔══██╗████╗ ████║██╔════╝      ██╔═══██╗██║   ██║██╔════╝██╔══██╗\n\
██║ ████║███████║██╔████╔██║█████╗        ██║   ██║██║   ██║█████╗  ██████╔╝\n\
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝        ██║   ██║██║   ██║██╔══╝  ██╔══██╗\n\
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗      ╚██████╔╝╚██████╔╝███████╗██║  ██║\n\
 ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝       ╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝\n\
                       Press R to restart the game ! \n\
                       Press M to go back to menu ! \n\
                       Press Q to quit ! ";

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

pub fn game_over_paragraph(frame: &mut Frame) {
    render_full_centered_paragraph(frame, GAME_OVER_TEXT, Some(Color::Red));
}
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
