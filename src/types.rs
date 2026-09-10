use colored::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct Barricade {
    pub team: Team,
    pub orientation: Orientation,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Team {
    Red,
    Blue,
}

impl From<Team> for Color {
    fn from(team: Team) -> Self {
        match team {
            Team::Red => Color::Red,
            Team::Blue => Color::Blue,
        }
    }
}
