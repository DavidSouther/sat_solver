use std::{f32::consts::PI, iter::FromIterator};

// ECEF Vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Position {
        Position { x, y, z }
    }

    pub fn len(&self) -> f32 {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        f32::sqrt(x * x + y * y + z * z)
    }

    pub fn sub(&self, a: &Position) -> Position {
        Position {
            x: self.x - a.x,
            y: self.y - a.y,
            z: self.z - a.z,
        }
    }

    pub fn dot(a: &Position, b: &Position) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn angle(a: &Position, b: &Position) -> f32 {
        let d = Position::dot(a, b);
        let n = a.len() * b.len();
        let t = f32::acos(d / n);
        t
    }

    pub fn separation(&self, a: &Position, b: &Position) -> f32 {
        Position::angle(&a.sub(self), &b.sub(self))
    }

    pub fn norm(&self) -> Position {
        let n = self.len();
        self.scale(1.0 / n)
    }

    pub fn scale(&self, n: f32) -> Position {
        Position::new(self.x * n, self.y * n, self.z * n)
    }

    // Returns true if the target entity is within `angle` of self, looking
    // along a normal oriented at <0>.
    pub fn can_see(&self, target: &Position, angle: f32) -> bool {
        let t = Position::dot(&self.norm(), target);
        if t <= self.len() {
            return false;
        }
        let dt = self.scale(t);
        let h = dt.len();
        let x = dt.sub(target).len();
        let r = h * (angle * PI / 180.0).tan();
        x < r
    }
}

impl<'a> FromIterator<&'a str> for Position {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let x = iter.next().unwrap().parse::<f32>().unwrap();
        let y = iter.next().unwrap().parse::<f32>().unwrap();
        let z = iter.next().unwrap().parse::<f32>().unwrap();
        Position { x, y, z }
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_build_position() {
        let position = String::from("6371 0 0");
        let parts = position.split_whitespace();
        let position: Position = parts.collect();
        assert_eq!(position.x, 6371.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(position.z, 0.0);
    }

    #[test]
    fn test_position_arithmetic() {
        let p1 = Position::new(2.0, 3.0, 4.0);
        let p2 = Position::new(3.0, 4.0, 6.0);

        assert_eq!(p2.len(), 7.81025);

        assert_eq!(p2.sub(&p1), Position::new(1.0, 1.0, 2.0));

        assert_eq!(Position::dot(&p1, &p2), 42.0);
    }

    #[test]
    fn test_position_angle() {
        let p1 = Position::new(1.0, 0.0, 0.0);
        let p2 = Position::new(0.0, 1.0, 0.0);
        let p3 = Position::new(0.0, 0.0, 1.0);

        let o = Position {
            x: 1.0,
            y: 1.0,
            z: 0.0,
        };

        assert_eq!(Position::angle(&p1, &p1), 0.0);
        assert_eq!(Position::angle(&p1, &p2), PI / 2.0);
        assert_eq!(Position::angle(&p1, &p3), PI / 2.0);
        assert_eq!(Position::angle(&p1, &p3), PI / 2.0);
        assert_eq!(o.separation(&p1, &p2), PI / 2.0)
    }

    #[test]
    fn test_can_see() {
        let x = Position::new(1.0, 1.0, 0.0);
        let p1 = Position::new(2.0, 3.0, 0.0);
        let p2 = Position::new(-2.0, 3.0, 0.0);
        let p3 = Position::new(-2.0, -3.0, 0.0);
        let p4 = Position::new(0.0, 0.0, 0.0);

        assert_eq!(x.can_see(&p1, 45.0), true);
        assert_eq!(x.can_see(&p2, 45.0), false);
        assert_eq!(x.can_see(&p3, 45.0), false);
        assert_eq!(x.can_see(&p4, 45.0), false);
    }

    #[test]
    fn regression_can_see() {
        let s1 = Position::new(6921.0, 0.0, 0.0);
        let g1 = Position::new(-5324.437140094696, -3507.3891257286095, -170.3720276523595);
        assert_eq!(g1.can_see(&s1, 45.0), false);

        let s3 = Position::new(0.0, 0.0, 2.0);
        let g3 = Position::new(1.0, 0.0, 0.0);
        assert_eq!(g3.can_see(&s3, 45.0), false);

        let s2 = Position::new(6921.0, 0.0, 0.0);
        let g2 = Position::new(111.189278, 0.0, 6370.02978);
        assert_eq!(g2.can_see(&s2, 45.0), false);
    }
}
