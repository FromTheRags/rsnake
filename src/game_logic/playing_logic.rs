use crate::controls::direction::Direction;
use crate::controls::speed::Speed;
use crate::game_logic::fruits_manager::FruitsManager;
use crate::game_logic::high_score::{HighScore, HighScoreManager};
use crate::game_logic::state::{GameOverMenu, GameState, GameStatus};
use crate::graphics::sprites::fruit::Fruit;
use crate::graphics::sprites::map::Map;
use crate::graphics::sprites::snake_body::SnakeBody;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, trace_span, warn};

/// # Panics
/// if Arc panics while holding the resources (poisoning),
/// no recovery mechanism implemented better crash  
pub fn playing_logic_loop(
    direction: &Arc<RwLock<Direction>>,
    snake: &Arc<RwLock<SnakeBody>>,
    gs: &Arc<RwLock<GameState>>,
    carte: &Arc<RwLock<Map>>,
    fruits_manager: &Arc<RwLock<FruitsManager>>,
    (game_speed, snake_symbols, negative_size_fruits): (Speed, String, bool),
) {
    let mut gsc;
    debug!(fruits = ?fruits_manager.read().unwrap().get_fruits(), "Initial fruits on map");
    loop {
        let _span = trace_span!("logic_iteration").entered();
        let loop_start = Instant::now();
        //do not want to keep the lock for too long + cannot hold in the same thread 2 times the same hold
        // so match a clone or use a let
        gsc = gs.read().unwrap().status.clone();
        //dead snakes tell no tales, nor move :p
        match gsc {
            GameStatus::Playing => {
                let mut write_guard = snake.write().unwrap();
                //Move the snake!
                let position = write_guard.ramp(&direction.read().unwrap(), &carte.read().unwrap());
                debug!(?position, "Snake moved");
                // Check if we eat a fruit
                let fruits = fruits_manager.read().unwrap().eat_some_fruits(position);
                // fruits effects
                if let Some(fruits) = fruits {
                    debug!(count = fruits.len(), "Fruits eaten by snake");
                    let score_fruits = fruits.iter().map(Fruit::get_score).sum::<i32>();
                    let size_effect = fruits.iter().map(Fruit::get_grow_snake).sum::<i16>();
                    info!(?fruits, "Fruit characteristics");
                    // In classic-like mode, negative fruits are disabled.
                    if negative_size_fruits || size_effect > 0 {
                        write_guard.relative_size_change(size_effect);
                    }
                    //NB:Converting an u16 to an i32 is always safe in Rust because the range of u16 (0 to 65,535)
                    // fits entirely within the range of i32 (−2,147,483,648 to 2,147,483,647).
                    //So no need to do: speed_score_modifier.try_into().expect("too much")/match for conversion
                    let fruit_impact = score_fruits * i32::from(game_speed.score_modifier());
                    info!(
                        fruit_impact = fruit_impact,
                        "Fruit impact on score because of speed score modifier"
                    );
                    let mut state_guard_gs = gs.write().unwrap();
                    // to always keep the score positive and between bounds
                    if fruit_impact < 0 {
                        state_guard_gs.score = state_guard_gs
                            .score
                            .saturating_sub(fruit_impact.unsigned_abs());
                    } else {
                        state_guard_gs.score = state_guard_gs
                            .score
                            .saturating_add(fruit_impact.unsigned_abs());
                    }
                    info!(
                        score = state_guard_gs.score,
                        snake_size = write_guard.body.len(),
                        "Current game metrics"
                    );
                    fruits_manager.write().unwrap().replace_fruits(&fruits);
                    debug!(fruits = ?fruits_manager.read().unwrap().get_fruits(), "Current fruits on map");
                }
                // Check if the gamer will lose one life
                if write_guard.is_snake_eating_itself() {
                    //Ouch. You bite yourself
                    let mut state_guard = gs.write().unwrap();
                    if (state_guard.life) >= 1 {
                        state_guard.life -= 1;
                        warn!(
                            remaining_life = state_guard.life,
                            "Ouch! Did that taste good? You bit yourself!",
                        );
                    }
                    if state_guard.life == 0 {
                        error!(
                            score = state_guard.score,
                            "FATAL: Snake succumbed to its injuries. Game Over!"
                        );
                        //The GameOverMenu option will be used to store the user selection of what to do
                        state_guard.status = GameStatus::GameOver(GameOverMenu::default());

                        // Save High Score
                        if let Ok(manager) = HighScoreManager::new() {
                            let score_value: u32 = state_guard.score;
                            let rank = manager.save_score(&HighScore::new(
                                snake_symbols.clone(),
                                score_value,
                                //using Display implementation
                                game_speed.to_string(),
                            ));
                            state_guard.rank = rank.unwrap_or(None);
                        }
                    }
                }
            }
            GameStatus::Restarting => {
                //let some time for the restarting screen to appear
                sleep(Duration::from_secs(1));
                gs.write().unwrap().reset();
                snake.write().unwrap().reset();
                *direction.write().unwrap() = Direction::Right;
                fruits_manager.write().unwrap().reset();
                //graphical resize on rendering part (not really a game_logic constant)
            }
            GameStatus::ByeBye | GameStatus::Menu => break,
            GameStatus::Paused | GameStatus::GameOver(_) => {}
        }
        let elapsed = loop_start.elapsed();
        trace!(?elapsed, "Logic loop iteration finished");
        sleep(Duration::from_millis(game_speed.ms_value()));
    }
}
