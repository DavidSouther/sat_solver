use super::position::Position;
use std::{fmt, iter::FromIterator};

pub struct Scenario {
    users: Vec<Entity>,
    satellites: Vec<Satellite>,
    pub interferers: Vec<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    id: i32,
    position: Position,
}

pub struct Satellite {
    entity: Entity,
    beams: Vec<Beam>,
}

pub struct Beam {
    user: Entity,
    band: Band,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Band {
    A,
    B,
    C,
    D,
}

pub const BANDS: [Band; 4] = [Band::A, Band::B, Band::C, Band::D];

impl Entity {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    // Returns true if the target entity is within `angle` of self, looking
    // along a normal oriented at <0>.
    pub fn can_see(&self, target: &Entity, angle: f32) -> bool {
        self.position().can_see(target.position(), angle)
    }
}

impl<'a> FromIterator<&'a str> for Entity {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let id = iter.next().unwrap().parse::<i32>().unwrap();
        let position: Position = iter.collect();
        Entity { id, position }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Satellite {
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn beams(&self) -> &Vec<Beam> {
        &self.beams
    }

    pub fn beams_mut(&mut self) -> &mut Vec<Beam> {
        &mut self.beams
    }
}

impl<'a> FromIterator<&'a str> for Satellite {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let entity: Entity = iter.collect();
        Satellite {
            entity,
            beams: Vec::with_capacity(32),
        }
    }
}

impl fmt::Display for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.entity.id)
    }
}

impl Beam {
    pub fn new(user: Entity, band: Band) -> Beam {
        Beam { user, band }
    }

    pub fn user(&self) -> &Entity {
        &self.user
    }

    pub fn band(&self) -> Band {
        self.band
    }
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Band::A => "A",
                Band::B => "B",
                Band::C => "C",
                Band::D => "D",
            }
        )
    }
}

impl Scenario {
    pub fn from_str(s: &str) -> Scenario {
        s.lines().collect()
    }

    pub fn users(&self) -> &Vec<Entity> {
        &self.users
    }

    pub fn satellites_mut(&mut self) -> &mut Vec<Satellite> {
        &mut self.satellites
    }

    pub fn satellites(&self) -> &Vec<Satellite> {
        &self.satellites
    }

    pub fn interferers(&self) -> &Vec<Entity> {
        &self.interferers
    }

    fn make() -> Scenario {
        Scenario {
            // capacities from eighteen planes scenario
            users: Vec::with_capacity(2500),
            satellites: Vec::with_capacity(360),
            interferers: Vec::with_capacity(50),
        }
    }

    fn add_line(&mut self, line: &str) {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("user") => self.users.push(parts.collect()),
            Some("sat") => self.satellites.push(parts.collect()),
            Some("interferer") => self.interferers.push(parts.collect()),
            Some(_) => (),
            None => (),
        }
        ()
    }
}

impl FromIterator<String> for Scenario {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut scenario = Scenario::make();
        for line in iter {
            scenario.add_line(line.as_str())
        }
        scenario
    }
}

impl<'a> FromIterator<&'a str> for Scenario {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut scenario = Scenario::make();
        for line in iter {
            scenario.add_line(line)
        }
        scenario
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines: Vec<String> = self
            .satellites
            .iter()
            .flat_map(|satellite| {
                let mut id = 1;
                satellite.beams().iter().map(move |beam| {
                    let f = format!(
                        "sat {} beam {} user {} color {}",
                        satellite.entity().id(),
                        id,
                        beam.user(),
                        beam.band(),
                    );
                    id += 1;
                    f
                })
            })
            .collect();
        lines.sort_unstable();
        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_entity() {
        let entity = String::from("1 6371.1897 22.65 -123");
        let parts = entity.split_whitespace();
        let entity: Entity = parts.collect();
        let position = entity.position;
        assert_eq!(entity.id, 1);
        assert_eq!(position.x, 6371.1897);
        assert_eq!(position.y, 22.65);
        assert_eq!(position.z, -123.0);
    }

    const EXAMPLE_SCENARIO: &str =
        "# User on the equator at the Prime Meridian, satellite 550km overhead
user 1 6371 0 0
sat 1 6921 0 0

# Two users close to the North Pole, satellite 550km overhead
user 2 0 0 6371
user 3 111.189281412 0 6370.02966584
sat 2 0 0 6921

# Interferer satellite in GEO at 180 degrees West (opposite the Prime Meridian)
interferer 1 -42164 0 0
";

    #[test]
    fn test_build_scenario() {
        let scenario = Scenario::from_str(EXAMPLE_SCENARIO);
        assert_eq!(scenario.users.len(), 3);
        assert_eq!(scenario.satellites.len(), 2);
        assert_eq!(scenario.interferers.len(), 1);
    }
}
