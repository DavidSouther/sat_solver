use std::f32::consts::PI;

use crate::{
    position::Position,
    scenario::{Band, Beam, Entity, Satellite, Scenario, BANDS},
};

impl Scenario {
    // For each user, build a set of the satellites it can see.
    // Invert the set to a map of Satellite -> User[]
    // Pack:
    //     Until no changes
    //          For each band
    //              For each satellite, add the next user that is in bounds
    pub fn optimize(&mut self) {
        let interferers = &self.interferers().clone();
        let mut users = self.users().clone();
        let mut iteration = 0;
        loop {
            iteration += 1;
            let start_users = users.len();
            for band in &BANDS {
                let band = *band;
                for satellite in self.satellites_mut().iter_mut() {
                    for index in (0..users.len()).rev() {
                        let user = users[index];
                        if satellite.can_accept(&user, band, interferers) {
                            users.swap_remove(index);
                            satellite.beams_mut().push(Beam::new(user, band));
                        }
                    }
                }
            }
            let users_added = start_users - users.len();
            eprintln!("Loop {} added {} users", iteration, users_added);
            if users_added == 0 || users.len() == 0 {
                break;
            }
        }
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
                    ) <= 10.0 * PI / 180.0
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
                ) <= 20.0 * PI / 180.0
                // > 20* from non-starlink sats
            })
            .any(|s| s)
    }

    pub fn can_accept(&self, user: &Entity, band: Band, interferers: &Vec<Entity>) -> bool {
        //  32 beams per satellite
        if self.beams().len() >= 32 {
            false
        // 45 degree visibility
        } else if !user.can_see(self.entity(), 45.0) {
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
        assert_eq!(scenario.satellites()[0].beams().len(), 3);
        let output = format!("{}", scenario);
        assert_eq!(
            output,
            "sat 1 beam 1 user 3 color A
sat 1 beam 2 user 2 color A
sat 1 beam 3 user 1 color B"
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
sat 2 beam 1 user 2 color A
sat 2 beam 2 user 3 color B"
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
