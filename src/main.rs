use std::{
    collections::{BTreeMap, BTreeSet},
    f32::consts::PI,
    fmt,
    io::{self, BufRead},
    str::SplitWhitespace,
};

// ECEF Vector
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn len(&self) -> f32 {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        f32::sqrt(x * x + y * y + z * z)
    }

    fn sub(&self, a: &Position) -> Position {
        Position {
            x: self.x - a.x,
            y: self.y - a.y,
            z: self.z - a.z,
        }
    }

    fn dot(a: &Position, b: &Position) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    fn angle(a: &Position, b: &Position) -> f32 {
        let d = Position::dot(a, b);
        let n = a.len() * b.len();
        let t = f32::acos(d / n);
        t * 180.0 / PI
    }

    fn separation(&self, a: &Position, b: &Position) -> f32 {
        Position::angle(&a.sub(self), &b.sub(self))
    }
}

#[derive(Debug, Clone, Copy)]
struct Entity {
    id: i32,
    position: Position,
}

// Build a map of User IDs to a Set of satellites it can see. It can see
// satellites within 45' of its normal.
fn skymap(scenario: &Scenario) -> BTreeMap<i32, BTreeSet<i32>> {
    BTreeMap::new()
}

// Invert the skymap to a map of Satellites to a list of Users it can see
fn groundmap(skymap: &BTreeMap<i32, BTreeSet<i32>>) -> BTreeMap<i32, Vec<Entity>> {
    BTreeMap::new()
}

impl Satellite {
    fn in_bounds(&self, user: &Entity, band: &Band, interferers: &Vec<Entity>) -> bool {
        //  32 beams per satellite
        if self.beams.len() == 32 {
            false
        } else {
            // > 10* from other users on the band on the satellite
            let beam_intersection = self
                .beams
                .iter()
                .map(|beam| {
                    Position::separation(&self.entity.position, &user.position, &beam.user.position)
                        > 10.0
                })
                .any(|s| !s);
            if beam_intersection {
                false
            } else {
                // > 20* from non-starlink sats
                //     Stretch goal: including adjacent satellites
                interferers
                    .iter()
                    .map(|interferer| {
                        Position::separation(
                            &self.entity.position,
                            &user.position,
                            &interferer.position,
                        ) > 20.0
                    })
                    .any(|s| !s)
            }
        }
    }

    fn maybe_add_next_user(
        &mut self,
        users: &Vec<Entity>,
        band: &Band,
        interferers: &Vec<Entity>,
    ) -> i32 {
        let user = users
            .iter()
            .find(|user| self.in_bounds(user, band, interferers));
        match user {
            Some(user) => {
                self.beams.push(Beam {
                    user: *user,
                    band: *band,
                });
                1
            }
            None => 0,
        }
    }
}

// For each user, build a set of the satellites it can see.
// Invert the set to a map of Satellite -> User[]
// Pack:
//     Until no changes
//          For each band
//              For each satellite, add the next user that is in bounds
fn optimize(scenario: &mut Scenario) {
    let skymap = skymap(&scenario);
    let groundmap = groundmap(&skymap);
    let interferers = &scenario.interferers;
    let empty = &Vec::new();
    loop {
        let mut users_added = 0;
        for band in &BANDS {
            let band = *band;
            for satellite in scenario.satellites.iter_mut() {
                let users = groundmap.get(&satellite.entity.id).unwrap_or(empty);
                users_added += satellite.maybe_add_next_user(users, &band, interferers);
            }
        }
        if users_added == 0 {
            break;
        }
    }
}

struct Satellite {
    entity: Entity,
    beams: Vec<Beam>,
}

#[derive(Debug, Clone, Copy)]
enum Band {
    A,
    B,
    C,
    D,
}

const BANDS: [Band; 4] = [Band::A, Band::B, Band::C, Band::D];

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

impl fmt::Display for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.entity.id)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

struct Beam {
    user: Entity,
    band: Band,
}

fn main() {
    let stdin = io::stdin();
    let mut scenario = read_scenario(&mut stdin.lock());
    eprintln!(
        "Read scenario\n\t{} users\n\t{} sats\n\t{} interferrers",
        scenario.users.len(),
        scenario.satellites.len(),
        scenario.interferers.len(),
    );
    optimize(&mut scenario);
    print_scenario(scenario);
}

fn build_position(iter: &mut SplitWhitespace) -> Position {
    let x = iter.next().unwrap().parse::<f32>().unwrap();
    let y = iter.next().unwrap().parse::<f32>().unwrap();
    let z = iter.next().unwrap().parse::<f32>().unwrap();
    Position { x, y, z }
}

fn build_entity(iter: &mut SplitWhitespace) -> Entity {
    let id = iter.next().unwrap().parse::<i32>().unwrap();
    let position = build_position(iter);
    Entity { id, position }
}

fn build_satellite(iter: &mut SplitWhitespace) -> Satellite {
    let entity = build_entity(iter);
    Satellite {
        entity,
        beams: Vec::with_capacity(32),
    }
}

struct Scenario {
    users: Vec<Entity>,
    satellites: Vec<Satellite>,
    interferers: Vec<Entity>,
}

fn print_scenario(scenario: Scenario) -> () {
    for satellite in scenario.satellites {
        let mut id = 1;
        for beam in satellite.beams {
            println!(
                "sat {} beam {} user {} color {}",
                satellite.entity.id, id, beam.user, beam.band,
            );
            id += 1;
        }
    }
}

fn read_scenario(buf: &mut dyn BufRead) -> Scenario {
    let mut scenario = Scenario {
        // capacities from eighteen planes scenario
        users: Vec::with_capacity(2500),
        satellites: Vec::with_capacity(360),
        interferers: Vec::with_capacity(50),
    };

    for line in buf.lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("user") => scenario.users.push(build_entity(&mut parts)),
            Some("sat") => scenario.satellites.push(build_satellite(&mut parts)),
            Some("interferer") => scenario.interferers.push(build_entity(&mut parts)),
            Some(_) => continue,
            None => continue,
        }
    }

    scenario
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_position() {
        let position = String::from("6371 0 0");
        let mut parts = position.split_whitespace();
        let position = build_position(&mut parts);
        assert_eq!(position.x, 6371.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(position.z, 0.0);
    }

    #[test]
    fn test_build_entity() {
        let entity = String::from("1 6371.1897 22.65 -123");
        let mut parts = entity.split_whitespace();
        let entity = build_entity(&mut parts);
        let position = entity.position;
        assert_eq!(entity.id, 1);
        assert_eq!(position.x, 6371.1897);
        assert_eq!(position.y, 22.65);
        assert_eq!(position.z, -123.0);
    }

    #[test]
    fn test_build_scenario() {
        let scenario = "# User on the equator at the Prime Meridian, satellite 550km overhead
user 1 6371 0 0
sat 1 6921 0 0

# Two users close to the North Pole, satellite 550km overhead
user 2 0 0 6371
user 3 111.189281412 0 6370.02966584
sat 2 0 0 6921

# Interferer satellite in GEO at 180 degrees West (opposite the Prime Meridian)
interferer 1 -42164 0 0
";
        let scenario = read_scenario(&mut scenario.as_bytes());
        assert_eq!(scenario.users.len(), 3);
        assert_eq!(scenario.satellites.len(), 2);
        assert_eq!(scenario.interferers.len(), 1);
    }
}
