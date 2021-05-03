use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(dead_code)]
const ORIGIN: Position = Position {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

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

    pub fn angle_origin(a: &Position, b: &Position, o: &Position) -> f32 {
        Position::angle(&a.sub(o), &b.sub(o))
    }

    pub fn angle(a: &Position, b: &Position) -> f32 {
        let r = Position::dot(&a.norm(), &b.norm());
        // When close to 0, catch possible NaN
        if (r - 1.0).abs() < 0.01 {
            return 0.0;
        }
        let t = f32::acos(r);
        t
    }

    pub fn separation(&self, a: &Position, b: &Position) -> f32 {
        Position::angle(&a.sub(self), &b.sub(self))
    }

    pub fn norm(&self) -> Position {
        self.scale(1.0 / self.len())
    }

    pub fn scale(&self, n: f32) -> Position {
        Position::new(self.x * n, self.y * n, self.z * n)
    }

    // Check if `target` is visible at least `angle` degrees above the horizon.
    // The surface is at `self` position, with a normal coming from the origin.
    // This test checks if the target is within a cone of `angle` degrees, whose
    // point is centered at `self` and expands along the normal. The `target`
    // point is first projected onto the normal; if it is behind `self` on the
    // normal the point is "behind" dishy and can't be seen. Otherwise, it
    // checks if the distance from the point to the normal is less than the
    // radius of the cone of `angle` degrees at that point. However, since
    // angle is always 45 for this project, it simplifies to the radius being
    // equal to the height on the cone.
    #[allow(dead_code)]
    pub fn can_see_cone(&self, target: &Position) -> bool {
        let n = self.norm();
        let l = self.len();
        let t = Position::dot(&n, target);
        if t <= l {
            return false;
        }
        let dt = n.scale(t);
        let h = t - l;
        let x = dt.sub(target).len();
        x < h
    }

    pub fn can_see_sat(&self, target: &Position) -> bool {
        // A copy of the version used in evaluate. This approach was not my
        // first attempt, but I'm glad I took the time to break it apart and
        // understand it. From my approach, I was trying to see if the user
        // could see the satellite, which resulted in the cone-intersection
        // approach. The angle calculation in the evaluate script flips this,
        // by instead checking if the angle from the satellite to the origin via
        // the ground is more than 135 degrees (the obtuse of 45).
        let r = Position::angle_origin(&ORIGIN, target, self);

        let k = r * 180.0 / PI;
        k > 180.0 - 45.0
    }

    pub fn can_see(&self, target: &Position) -> bool {
        self.can_see_sat(target)
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
        let o = Position::new(1.0, 1.0, 0.0);

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

        assert_eq!(x.can_see(&p1), true);
        assert_eq!(x.can_see(&p2), false);
        assert_eq!(x.can_see(&p3), false);
        assert_eq!(x.can_see(&p4), false);
    }

    #[test]
    fn regression_can_see() {
        // Initial can_see code was problematic; several iterations with these
        // inputs got it there.
        let s1 = Position::new(6921.0, 0.0, 0.0);
        let g1 = Position::new(-5324.437140094696, -3507.3891257286095, -170.3720276523595);
        assert_eq!(g1.can_see(&s1), false);

        let s3 = Position::new(0.0, 0.0, 2.0);
        let g3 = Position::new(1.0, 0.0, 0.0);
        assert_eq!(g3.can_see(&s3), false);

        let s2 = Position::new(6921.0, 0.0, 0.0);
        let g2 = Position::new(111.189278, 0.0, 6370.02978);
        assert_eq!(g2.can_see(&s2), false);

        let s4 = Position::new(6921.0, 0.0, 0.0);
        let g4 = Position::new(6350.206256636249, 574.1605965872963, -160.24555276741216);
        assert_eq!(g4.can_see(&s4), false);
    }

    #[test]
    fn regression_angle() {
        // This specific pair of ground stations triggerd an f32 precision error
        // in f32::cos, so there's a special case in angle for close to 0*
        let s = Position::new(-5111.007144121957, -1334.7360828140702, 4471.7252332817225);
        let p1 = Position::new(-4462.399898375494, -1507.4791341925356, 4286.176851267787);
        let p2 = Position::new(-4462.341423785467, -1507.5185635095902, 4286.223546883291);

        let separation = s.separation(&p1, &p2);
        assert_eq!(separation, 0.0);
    }
}
