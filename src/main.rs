use std::io::{self, BufRead};

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
}
