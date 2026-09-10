use std::io::stdin;

use crate::{
    bot::{Bot, Context},
    coord::PCoord,
    game::Action::Move,
    map::Map,
    types::{Orientation, Team},
};

pub enum Action {
    Move(PCoord),
    Barricade(PCoord, Orientation),
}

impl Action {
    pub fn parse(input: impl AsRef<str>) -> Option<Action> {
        let trimmed_input = input.as_ref().trim().to_ascii_lowercase();
        let mut chars = trimmed_input.chars();

        let c1 = chars.next()?;
        let c2 = chars.next()?;

        if let Some(c3) = chars.next() {
            // 3 chars => place barricade
            let x = (c2.to_ascii_lowercase() as i32) - i32::from(b'a');
            let y = (c3 as i32) - i32::from(b'1');
            let coord = PCoord::new(x, y);

            let orientation = match c1 {
                'v' => Some(Orientation::Vertical),
                'h' => Some(Orientation::Horizontal),
                _ => None,
            }?;

            return Some(Action::Barricade(coord, orientation));
        } else {
            // 2 chars => move
            let x = (c1.to_ascii_lowercase() as i32) - i32::from(b'a');
            let y = (c2 as i32) - i32::from(b'1');
            let coord = PCoord::new(x, y);

            return Some(Move(coord));
        }
    }
}

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

    fn read_action_from_stdin() -> Option<Action> {
        let mut input = String::new();
        stdin().read_line(&mut input).ok()?;
        Action::parse(input)
    }

    fn execute_action(&mut self, action: Action, team: Team) -> bool {
        let player = match team {
            Team::Red => &mut self.red_player,
            Team::Blue => &mut self.blue_player,
        };

        match action {
            Action::Move(coord) => self.map.try_step(player.team, coord).is_ok(),
            Action::Barricade(coord, orientation) => {
                if player.inventory > 0
                    && self
                        .map
                        .try_place_barricade(player.team, coord.to_bcoord(), orientation)
                        .is_ok()
                {
                    player.inventory -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn pvp(&mut self) {
        self.play(
            Box::new(|_| Self::read_action_from_stdin()),
            Box::new(|_| Self::read_action_from_stdin()),
        );
    }

    pub fn pve(&mut self, mut bot: impl Bot) {
        self.play(
            Box::new(|_| Self::read_action_from_stdin()),
            Box::new(move |ctx| Some(bot.get_action(ctx))),
        );
    }

    pub fn play<'a>(
        &mut self,
        mut red: Box<dyn FnMut(Context) -> Option<Action> + 'a>,
        mut blue: Box<dyn FnMut(Context) -> Option<Action> + 'a>,
    ) {
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
            let action_provider = if turn % 2 == 0 { &mut red } else { &mut blue };

            let context = Context::new(
                self.map.clone(),
                self.blue_player.inventory,
                self.red_player.inventory,
            );

            if let Some(action) = action_provider(context) {
                let team = if turn % 2 == 0 { Team::Red } else { Team::Blue };
                let ok = self.execute_action(action, team);

                if ok {
                    turn += 1;
                }
            }
        }
    }
}
