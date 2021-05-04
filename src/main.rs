use std::io::{self, BufRead};

#[cfg(feature = "analysis")]
use std::fmt;

mod position;
mod scenario;
mod solver;

use crate::scenario::Scenario;

fn main() {
    let stdin = io::stdin();
    let buf = &mut stdin.lock();
    let mut scenario: Scenario = buf
        .lines()
        .map(|line| line.unwrap_or(String::from("")))
        .collect();

    eprintln!(
        "Read scenario\n\t{} users\n\t{} sats\n\t{} interferrers",
        scenario.users().len(),
        scenario.satellites().len(),
        scenario.interferers().len(),
    );

    scenario.optimize();
    println!("{}", scenario);

    #[cfg(analysis)]
    {
        let analysis = scenario.analyze();
        eprintln!("Analysis: ");
        eprintln!("{}", analysis);
    }
}

#[cfg(feature = "analysis")]
struct Analysis<'a> {
    scenario: &'a Scenario,
    saturated: i32,
    unassigned: i32,
    max_possible: usize,           // Number of satellites * 32
    max_possible_utilization: f32, // max_possible / number of users
    uncovered: usize,
    max_visible_utilization: f32, // max_possible / (number of users - uncovered users)
}

#[cfg(feature = "analysis")]
impl<'a> Analysis<'a> {
    // % of max utilization achieved;
    fn success(&self) -> f32 {
        let best_utilization = self
            .max_visible_utilization
            .min(self.max_possible_utilization);
        self.scenario.utilization() / best_utilization
    }
}

#[cfg(feature = "analysis")]
impl Scenario {
    // % of users assigned
    fn utilization(&self) -> f32 {
        self.assigned as f32 / self.users().len() as f32
    }

    fn analyze(&self) -> Analysis {
        let mut analysis = Analysis {
            scenario: self,
            saturated: 0,
            unassigned: 0,
            max_possible: 0,
            max_possible_utilization: 0.0,
            uncovered: 0,
            max_visible_utilization: 0.0,
        };

        // Count number of saturated satelites, number of unassigned satellites,
        // and maximum satellite utilization.
        self.satellites()
            .iter()
            .for_each(|s| match s.beams().len() {
                32 => analysis.saturated += 1,
                0 => analysis.unassigned += 1,
                _ => (),
            });

        analysis.uncovered = self
            .users()
            .iter()
            .map(|u| u.position())
            .filter(|u| {
                self.satellites()
                    .iter()
                    .map(|s| s.entity().position())
                    .filter(|s| u.can_see(s))
                    .count()
                    == 0
            })
            .count();

        analysis.max_possible = (self.satellites().len() * 32).min(self.users().len());
        analysis.max_possible_utilization =
            analysis.max_possible as f32 / self.users().len() as f32;
        analysis.max_visible_utilization = ((self.users().len() - analysis.uncovered) as f32
            / analysis.max_possible as f32)
            .min(1.0);

        analysis
    }
}

#[cfg(feature = "analysis")]
impl<'a> fmt::Display for Analysis<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\t{} satellites are saturated", self.saturated)?;
        writeln!(f, "\t{} satellites are unassigned", self.unassigned)?;
        writeln!(
            f,
            "\tA most, {} users can be served ({:4}%)",
            self.max_possible,
            self.max_possible_utilization * 100.0
        )?;
        writeln!(
            f,
            "\t\tCurrent solution covered {} ({:4}%)",
            self.scenario.assigned,
            self.scenario.utilization() * 100.0
        )?;
        writeln!(f, "\t{} users are completeley uncovered", self.uncovered)?;
        writeln!(
            f,
            "\tSolution is overall {:4}% successful, given visiblity and capacity",
            self.success() * 100.0
        )?;
        Ok(())
    }
}
