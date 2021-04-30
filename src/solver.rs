use std::collections::{BTreeMap, BTreeSet};

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

impl Satellite {
    fn beam_intersection(&self, user: &Entity, band: Band) -> bool {
        self.beams()
            .iter()
            .map(|beam| {
                beam.band() == band
                    && Position::separation(
                        self.entity().position(),
                        user.position(),
                        beam.user().position(),
                    ) <= 10.0
            })
            .any(|s| s)
    }

    fn interference(&self, user: &Entity, interferers: &Vec<Entity>) -> bool {
        interferers
            .iter()
            .map(|interferer| {
                Position::separation(
                    user.position(),
                    self.entity().position(),
                    interferer.position(),
                ) <= 20.0
                // > 20* from non-starlink sats
            })
            .any(|s| s)
    }

    fn in_bounds(&self, user: &Entity, band: Band, interferers: &Vec<Entity>) -> bool {
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

    fn maybe_add_next_user(
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
}

// For each user, build a set of the satellites it can see.
// Invert the set to a map of Satellite -> User[]
// Pack:
//     Until no changes
//          For each band
//              For each satellite, add the next user that is in bounds
pub fn optimize(scenario: &mut Scenario) {
    let skymap = skymap(&scenario);
    let groundmap = groundmap(&skymap);
    let empty = &Vec::new();
    loop {
        let mut users_added = 0;
        for band in &BANDS {
            let band = *band;
            for satellite in scenario.satellites_mut().iter_mut() {
                let users = groundmap.get(&satellite.entity().id()).unwrap_or(empty);
                users_added += satellite.maybe_add_next_user(users, band, &vec![]);
                // &scenario.interferers);
            }
        }
        if users_added == 0 {
            break;
        }
    }
}
