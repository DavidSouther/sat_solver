# Sat Solver

This code implements a naive O(N^2) beam planning algorithm for the
specification provided in the Text Test pdf - namely, beams are assigned to user
terminal ground stations obeying certain physical and operational rules.

The code is written in Rust, primarily to leverage its fast iterator
implementations. (But really I just wanted to use a language I'm less familiar
with.) It expects the data file on standard input (because why not). The
included `run.sh` file will run all the scripts in the test_cases file, passing
them through the evaluator. A sample run is in `sample.txt`, and is summarized
below. It was exected with `cargo --version cargo 1.51.0 (43b129a20 2021-03-16)`
but will presumably work on any version later than 1.35 (that is, capable of
handling edition 2018). The code has no other dependencies.

The `src/` directory is split into four files. `main.rs` has a single `main`
method to collect stdin into a scenario, optimize the scenario, and print the
solution to stdout (with some analytics on stderr). `position.rs` contains a
vec3 implementation with the methods necessary for this project; it is certainly
not a full featured linear algebra library, and in production systems should be
replaced with an appropriate vec3d crate. `scenario.rs` contains several structs
holding the necessary pieces of the problem, and has some utility
implementations for interfacing with input, formatting, and `Position`. The
final piece is `solver`, which adds an `optimize` method to `Scenario`, as well
as utilities for packing and constraints. Comments on the algorithm are inline.

## Sample Run Summary
    Running on a Github Codespaces VM; 2.7Ghz 4 core CPU / 8GB ram

    Running 00_example.txt
    100.0% of 3 total users covered.
    Solution passed all checks!

    Running 01_simplest_possible.txt
    100.0% of 1 total users covered.
    Solution passed all checks!

    Running 02_two_users.txt
    100.0% of 2 total users covered.
    Solution passed all checks!

    Running 03_five_users.txt
    80.0% of 5 total users covered.
    Solution passed all checks!

    Running 04_one_interferer.txt
    0.0% of 1 total users covered.
    Solution passed all checks!

    Running 05_equatorial_plane.txt
    100.0% of 1000 total users covered.
    Solution passed all checks!

    Running 06_partially_fullfillable.txt
    76.68% of 2500 total users covered.
    Solution passed all checks!

    Running 07_eighteen_planes.txt
    98.44000000000001% of 2500 total users covered.
    Solution passed all checks!

    Running 08_eighteen_planes_northern.txt
    79.08% of 2500 total users covered.
    Solution passed all checks!

    Running 09_ten_thousand_users.txt
    91.94% of 10000 total users covered.
    Solution passed all checks!

    Running 10_ten_thousand_users_geo_belt.txt
    82.39999999999999% of 10000 total users covered.
    Solution passed all checks!

    Running 11_one_hundred_thousand_users.txt
    29.250999999999998% of 100000 total users covered.
    Solution passed all checks!

    time run.sh
    real    0m46.404s
    user    0m46.944s
    sys     0m0.328s
