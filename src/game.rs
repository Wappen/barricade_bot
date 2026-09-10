use std::{io::stdin, thread::sleep, time::Duration};

use crate::{
    bot::{Action, Bot, Context},
    coord::PCoord,
    map::Map,
    types::{Orientation, Team},
};

pub struct Player {
    team: Team,
    inventory: usize, // number of barricades left in inventory
}

impl Player {
    pub fn new(team: Team) -> Self {
        Self {
            team,
            inventory: 10,
        }
    }
}

pub struct Game {
    map: Map,
    red_player: Player,
    blue_player: Player,
}

impl Game {
    pub fn new() -> Self {
        let red_player = Player::new(Team::Red);
        let blue_player = Player::new(Team::Blue);

        Self {
            map: Map::new(PCoord::new(4, 0), PCoord::new(4, 8)),
            red_player,
            blue_player,
        }
    }

    pub fn run(&mut self) {
        let mut turn = 0;

        loop {
            println!("Red: {}", self.red_player.inventory);
            println!("Blue: {}", self.blue_player.inventory);
            println!("{}", self.map);

            if self.map.blue_coord.y == 0 {
                // blue wins
                println!("Blue wins!");
                return;
            }
            if self.map.red_coord.y == 8 {
                // red wins
                println!("Red wins!");
                return;
            }

            println!("{}'s turn", if turn % 2 == 0 { "Red" } else { "Blue" });

            let mut input = String::new();
            if stdin().read_line(&mut input).is_err() {
                continue;
            }

            let input = input.trim();
            let mut chars = input.chars();

            let Some(cmd) = chars.next() else {
                continue;
            };
            let Some(c1) = chars.next() else {
                continue;
            };
            let Some(c2) = chars.next() else {
                continue;
            };

            let x = (c1.to_ascii_lowercase() as i32) - i32::from(b'a');
            let y = (c2 as i32) - i32::from(b'1');
            let coord = PCoord::new(x, y);

            let current_player = if turn % 2 == 0 {
                &mut self.red_player
            } else {
                &mut self.blue_player
            };

            let ok = match cmd.to_ascii_lowercase() {
                'm' => self.map.try_step(current_player.team, coord).is_ok(),
                'v' | 'h' => {
                    let orientation = if cmd.to_ascii_lowercase() == 'v' {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    };

                    if current_player.inventory > 0
                        && self
                            .map
                            .try_place_barricade(
                                current_player.team,
                                coord.to_bcoord(),
                                orientation,
                            )
                            .is_ok()
                    {
                        current_player.inventory -= 1;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if ok {
                turn += 1;
            }
        }
    }

    pub fn pve<B: Bot>(&mut self, mut bot: B) {
        let mut turn = 0;

        loop {
            println!("Red: {}", self.red_player.inventory);
            println!("Blue: {}", self.blue_player.inventory);
            println!("{}", self.map);

            if self.map.blue_coord.y == 0 {
                // blue wins
                println!("Blue wins!");
                return;
            }
            if self.map.red_coord.y == 8 {
                // red wins
                println!("Red wins!");
                return;
            }

            println!("{}'s turn", if turn % 2 == 0 { "Red" } else { "Blue" });

            let ok = if turn % 2 == 0 {
                // player's turn
                let mut input = String::new();
                if stdin().read_line(&mut input).is_err() {
                    continue;
                }

                let input = input.trim();
                let mut chars = input.chars();

                let Some(cmd) = chars.next() else {
                    continue;
                };
                let Some(c1) = chars.next() else {
                    continue;
                };
                let Some(c2) = chars.next() else {
                    continue;
                };

                let x = (c1.to_ascii_lowercase() as i32) - i32::from(b'a');
                let y = (c2 as i32) - i32::from(b'1');
                let coord = PCoord::new(x, y);

                let current_player = if turn % 2 == 0 {
                    &mut self.red_player
                } else {
                    &mut self.blue_player
                };

                match cmd.to_ascii_lowercase() {
                    'm' => self.map.try_step(current_player.team, coord).is_ok(),
                    'v' | 'h' => {
                        let orientation = if cmd.to_ascii_lowercase() == 'v' {
                            Orientation::Vertical
                        } else {
                            Orientation::Horizontal
                        };

                        if current_player.inventory > 0
                            && self
                                .map
                                .try_place_barricade(
                                    current_player.team,
                                    coord.to_bcoord(),
                                    orientation,
                                )
                                .is_ok()
                        {
                            current_player.inventory -= 1;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                // bot's turn
                let context = Context::new(
                    self.map.clone(),
                    self.blue_player.inventory,
                    self.red_player.inventory,
                );

                sleep(Duration::from_secs(1));

                let action = bot.get_action(context);

                let current_player = if turn % 2 == 0 {
                    &mut self.red_player
                } else {
                    &mut self.blue_player
                };

                match action {
                    Action::Move(coord) => self.map.try_step(current_player.team, coord).is_ok(),
                    Action::Barricade(coord, orientation) => {
                        if current_player.inventory > 0
                            && self
                                .map
                                .try_place_barricade(
                                    current_player.team,
                                    coord.to_bcoord(),
                                    orientation,
                                )
                                .is_ok()
                        {
                            current_player.inventory -= 1;
                            true
                        } else {
                            false
                        }
                    }
                }
            };

            if ok {
                turn += 1;
            }
        }
    }
}
