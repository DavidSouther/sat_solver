use std::{
    collections::{BTreeMap, BTreeSet},
    f32::consts::PI,
};

use crate::{
    position::Position,
    scenario::{Band, Beam, Entity, Satellite, Scenario, BANDS},
};

// Build a map of User IDs to a Set of satellites it can see. It can see
// satellites within 45' of its normal.
fn skymap(scenario: &Scenario) -> BTreeMap<i32, BTreeSet<i32>> {
    BTreeMap::new()
}

// Invert the skymap to a map of Satellites to a list of Users it can see
fn groundmap(skymap: &BTreeMap<i32, BTreeSet<i32>>) -> BTreeMap<i32, Vec<Entity>> {
    BTreeMap::new()
}

impl Scenario {
    // For each user, build a set of the satellites it can see.
    // Invert the set to a map of Satellite -> User[]
    // Pack:
    //     Until no changes
    //          For each band
    //              For each satellite, add the next user that is in bounds
    pub fn optimize(&mut self) {
        let no_users: &Vec<Entity> = &Vec::new();
        let interferers = &self.interferers().clone();
        let skymap = skymap(self);
        // All users are now owned by groundmap
        let groundmap = groundmap(&skymap);
        loop {
            let mut users_added = 0;
            for band in &BANDS {
                let band = *band;
                for satellite in self.satellites_mut().iter_mut() {
                    let users = groundmap.get(&satellite.entity().id()).unwrap_or(no_users);
                    users_added += satellite.maybe_add_next_user(users, band, interferers);
                }
            }
            if users_added == 0 {
                break;
            }
        }
    }
}

impl Satellite {
    pub fn maybe_add_next_user(
        &mut self,
        users: &Vec<Entity>,
        band: Band,
        interferers: &Vec<Entity>,
    ) -> i32 {
        let user = users
            .iter()
            .find(|user| self.in_bounds(user, band, interferers));
        match user {
            Some(user) => {
                self.beams_mut().push(Beam::new(*user, band));
                1
            }
            None => 0,
        }
    }

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

    pub fn in_bounds(&self, user: &Entity, band: Band, interferers: &Vec<Entity>) -> bool {
        //  32 beams per satellite
        if self.beams().len() >= 32 {
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
    fn test_maybe_add_user() {
        let mut scenario = Scenario::from_str(
            "user 1 6371 0 0
user 2 6371 10 0
user 3 6371 400 400
sat 1 6921 0 0",
        );
        let users = &scenario.users().clone();
        let interferers = &Vec::new();
        let satellite = scenario.satellites_mut().iter_mut().next().unwrap();
        satellite.maybe_add_next_user(users, Band::A, interferers);
        satellite.maybe_add_next_user(users, Band::A, interferers);
        satellite.maybe_add_next_user(users, Band::A, interferers);
        assert_eq!(scenario.satellites()[0].beams().len(), 2);
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
        assert_eq!(scenario.satellites()[0].beams().len(), 2);
    }
}
