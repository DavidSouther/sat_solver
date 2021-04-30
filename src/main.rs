use std::{
    io::{self, BufRead},
    str::SplitWhitespace,
};

// ECEF Vector
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct Entity {
    id: i32,
    position: Position,
}

fn main() {
    let stdin = io::stdin();
    let scenario = read_scenario(&mut stdin.lock());
    println!(
        "Read scenario\n\t{} users\n\t{} sats\n\t{} interferrers",
        scenario.users.len(),
        scenario.satellites.len(),
        scenario.interferers.len(),
    )
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

struct Scenario {
    users: Vec<Entity>,
    satellites: Vec<Entity>,
    interferers: Vec<Entity>,
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
            Some("sat") => scenario.satellites.push(build_entity(&mut parts)),
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
