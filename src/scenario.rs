use super::position::Position;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    iter::FromIterator,
};

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
    visible: Vec<Entity>,
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

    pub fn visible(&self) -> &Vec<Entity> {
        &self.visible
    }

    pub fn maybe_see_next_user(&mut self, user: &Entity) -> () {
        if user.can_see(self.entity(), 45.0) {
            self.visible.push(user.clone())
        }
    }
}

impl<'a> FromIterator<&'a str> for Satellite {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let entity: Entity = iter.collect();
        Satellite {
            entity,
            beams: Vec::with_capacity(32),
            visible: Vec::with_capacity(64),
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

    pub fn users_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.users
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

    pub fn build_skymap(&mut self) {
        let users = self.users().clone();
        users.iter().for_each(|user| {
            self.satellites_mut()
                .iter_mut()
                .for_each(|s| s.maybe_see_next_user(user))
        })
    }

    // Build a map of User IDs to a Set of satellites it can see. It can see
    // satellites within 45' of its normal.
    //
    // Can be improved by first putting the satellites in a quadtree, is
    // currently O(n^2)
    pub fn skymap(&self) -> BTreeMap<i32, BTreeSet<i32>> {
        self.users()
            .iter()
            .map(|user| {
                (
                    user.id(),
                    self.satellites()
                        .iter()
                        .filter(|satellite| user.can_see(satellite.entity(), 45.0))
                        .map(|s| s.entity().id())
                        .collect(),
                )
            })
            .collect()
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
        let lines: Vec<String> = self
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

    #[test]
    fn test_scenario_skymap() {
        let mut scenario = Scenario::from_str(
            "user 1 6371 0 0
user 2 6371 10 0
user 3 6371 400 400
sat 1 6921 0 0
user 4 0 0 6371
user 5 111.189281412 0 6370.02966584
sat 2 0 0 6921
",
        );
        scenario.build_skymap();
        assert_eq!(scenario.satellites()[0].visible().len(), 2);
        assert_eq!(scenario.satellites()[1].visible().len(), 2);
    }
}
