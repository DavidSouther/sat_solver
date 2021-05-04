#[cfg(feature = "elevation")]
use std::cmp::Ordering;

#[cfg(feature = "elevation")]
use crate::position::ORIGIN;
use crate::{
    position::Position,
    scenario::{Band, Beam, Entity, Satellite, Scenario, BANDS},
};

impl Scenario {
    // Simple first-come-first serve packing. It hops across the bands, pulling
    // users from a queue and assigning them to the next best satellite. The
    // best satellite comes from find_best, below. Overall, this algorithm is
    // O(Users * Satellites), and could be improved by improving the runtime of
    // find_best. However, that algorithmic complexity is greatly mitigated by
    // the stop-quick nature of Rust iterators, which will return as soon as a
    // satellite is found. Still, it is worst case O(N^2).
    //
    // This was originally written to loop until no changes occured, but in
    // practice, the second iteration was never able to improve packing. This
    // seems to be because the satellites filled up quite quickly. Removing the
    // second iteration improves runtime.
    //
    // Future explorations would include improving the runtime of find_best
    // perhaps by using a QuadTree sharded on lat/long of the satellite, or
    // improving the packing efficiency for the 100k users test. However, that
    // may required a more exhaustive search approach, including backtracking
    // and spilling users between satellites. This naive approach works in a
    // suprisingly (to me) good manner.
    pub fn optimize(&mut self) {
        let interferers = &self.interferers().clone();
        let mut users = self.users().clone();
        let start_users = users.len();
        BANDS.iter().for_each(|band| {
            for index in (0..users.len()).rev() {
                let user = users[index];
                self.find_best(&user, *band, interferers).and_then(|s| {
                    users.swap_remove(index);
                    s.beams_mut().push(Beam::new(user, *band));
                    Some(())
                });
            }
        });
        self.assigned = start_users - users.len();
        eprintln!("Assigned {} users", self.assigned);
    }

    #[cfg(feature = "first")]
    pub fn find_best(
        &mut self,
        user: &Entity,
        band: Band,
        interferers: &Vec<Entity>,
    ) -> Option<&mut Satellite> {
        self.satellites_mut()
            .iter_mut()
            .filter(|s| s.can_accept(&user, band, interferers))
            .next()
    }

    // Find the next best satellite for the user.
    #[cfg(feature = "elevation")]
    pub fn find_best(
        &mut self,
        user: &Entity,
        band: Band,
        interferers: &Vec<Entity>,
    ) -> Option<&mut Satellite> {
        let position = &user.position();
        self.satellites_mut()
            .iter_mut()
            .filter(|s| s.can_accept(&user, band, interferers))
            .max_by(|a, b| {
                let angle_a = Position::angle_origin(&ORIGIN, a.entity().position(), position);
                let angle_b = Position::angle_origin(&ORIGIN, b.entity().position(), position);
                // angleA.cmp(angleB)
                if angle_a < angle_b {
                    Ordering::Less
                } else if angle_a > angle_b {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
    }
}

impl Satellite {
    pub fn beam_intersection(&self, user: &Entity, band: Band) -> bool {
        self.beams()
            .iter()
            .map(|beam| {
                beam.band() == band
                    && Position::separation(
                        self.entity().position(),
                        user.position(),
                        beam.user().position(),
                    )
                    .to_degrees()
                        <= 10.0
            })
            .any(|s| s)
    }

    pub fn interference(&self, user: &Entity, interferers: &Vec<Entity>) -> bool {
        interferers
            .iter()
            .map(|interferer| {
                Position::separation(
                    user.position(),
                    self.entity().position(),
                    interferer.position(),
                )
                .to_degrees()
                    <= 20.0
                // > 20* from non-starlink sats
            })
            .any(|s| s)
    }

    pub fn can_accept(&self, user: &Entity, band: Band, interferers: &Vec<Entity>) -> bool {
        //  32 beams per satellite
        if self.beams().len() >= 32 {
            false
        // 45 degree visibility
        } else if !user.position().can_see(self.entity().position()) {
            false
        } else if self.beam_intersection(user, band) {
            false
        } else if self.interference(user, interferers) {
            false
            // Stretch goal: including adjacent satellites
        } else {
            true
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_beam_intersection() {
        let mut scenario = Scenario::from_str(
            "user 1 6371 0 0
user 2 6371 10 0
user 3 6371 400 400
sat 1 6921 0 0",
        );
        let user = scenario.users()[0];
        scenario.satellites_mut()[0]
            .beams_mut()
            .push(Beam::new(user, Band::A));
        let satellite = &scenario.satellites()[0];
        assert_eq!(
            satellite.beam_intersection(&scenario.users()[1], Band::A),
            true
        );
        assert_eq!(
            satellite.beam_intersection(&scenario.users()[2], Band::A),
            false
        );
    }

    #[test]
    fn test_optimize() {
        let mut scenario = Scenario::from_str(
            "user 1 6371 0 0
user 2 6371 10 0
user 3 6371 400 400
sat 1 6921 0 0",
        );
        scenario.optimize();
        let output = format!("{}", scenario);
        assert_eq!(
            output,
            "sat 1 beam 1 user 2 color A
sat 1 beam 2 user 1 color B"
        );
    }

    #[test]
    fn test_00_example() {
        let mut scenario = Scenario::from_str(
            "user 1 6371 0 0
sat 1 6921 0 0
user 2 0 0 6371
user 3 111.189281412 0 6370.02966584
sat 2 0 0 6921
interferer 1 -42164 0 0
",
        );

        scenario.optimize();
        let output = format!("{}", scenario);

        assert_eq!(
            output,
            "sat 1 beam 1 user 1 color A
sat 2 beam 1 user 3 color A
sat 2 beam 2 user 2 color A"
        );
    }

    #[test]
    fn test_11_example_beam_interference() {
        /*
        Checking no sat interferes with itself...
        Sat 624 beams 2 and 27 interfere.
                Beam angle: 0.005501014798818914 degrees.
        sat 624 beam 2 user 83636 color A
        sat 624 beam 27 user 21283 color A
        sat 624 -5111.007144121957 -1334.7360828140702 4471.7252332817225
        user 83636 -4462.399898375494 -1507.4791341925356 4286.176851267787
        user 21283 -4462.341423785467 -1507.5185635095902 4286.223546883291
        */
        let mut scenario = Scenario::from_str(
            "sat 624 -5111.007144121957 -1334.7360828140702 4471.7252332817225
user 83636 -4462.399898375494 -1507.4791341925356 4286.176851267787
user 21283 -4462.341423785467 -1507.5185635095902 4286.223546883291",
        );

        let user = scenario.users()[0];
        scenario.satellites_mut()[0]
            .beams_mut()
            .push(Beam::new(user, Band::A));
        let satellite = &scenario.satellites()[0];
        let user = &scenario.users()[1];
        let intersects = satellite.beam_intersection(user, Band::A);
        assert_eq!(intersects, true);
    }
}
