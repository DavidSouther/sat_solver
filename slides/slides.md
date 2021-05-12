---
marp: true
---

# Beam Planner
David Souther

---

## Core Algorithm

* Packing optimization problem, NP complete
* Naive solution is simple packing, Works surprisingly well
* `find_best_satellite` encapsulates and isolates the hard\* part
* Loop until no changes to user set 
  * For each color band
    * For each user in set 
      * Find first eligible satellite
        * If there is an elligible satellite
          * Assign user to next beam & remove from set
* \* There's a second hard part that could involve "spilling" from one satellite to another, see optimizations later

---

## Technology Considerations

*More Important*

* Fast iteration
* Readable constructs

_Less Important_

* Memory Safety
* Threading

---

## Technology Decision (1/2)

Shortlist of languages I have spent more than ~10 minutes with in the past year

* Python
  * \+ Iterators
  * \+ Readable
  * \- Some memory & interpreter overhead
* Typescript
  * \~ Iterables, but mostly just arrays
  * \- Memory & interpreter overhead
  * \- Significant project setup
* Java
  * \- Makes TypeScript setup look like bash

---

## Technology Decision (2/2)

Shortlist of languages I have spent more than ~10 minutes with in the past year
* F#
  * \+ Iterables
  * \+ Legible Immutable structs are easy
  * \- dotnet either works or it doesn't
* Rust
  * \+ Fast Iterators
  * \+ Idiomatic Iterators
  * \+ Minimal memory overhead
  * \+ Trivial setup

Decision: Rust

---

## Supporting Components

* Algorithm needs to parse input
* Format output according to spec
* Measure angles between points in 3-space

---

### Parsing

* Data is well structured
* `#...` for comment lines
* `user ...` for users
* `sat ...` for Starlink
* `interferer ...` for others
* All entities are `id <position>`
* position is `float float float`

---

### [Read & Parse](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/scenario.rs#L123-L131)

* Get Lines over input stream
* Ignore lines starting with #
* Add other lines based on entity type 
* trait FromIterator makes creating an entity-specific parser dead simple
* Usage is brilliantly simple:
  * `a: FromIterType = iter<&str>.collect()`
  * [main.ts](https://github.com/DavidSouther/spacex_interview/blob/main/src/main.rs#L15-L18)

---

### [Formatting](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/scenario.rs#L163-L186)

* Once again, the Display trait is brilliant
* Better separation of concerns than overriding `toString` or implementing `__repr__`

---

### [Position Geometry](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/position.rs#L46-L54)

* As presented, "simple" spatial problems
* Beam angles 10&deg; minimum; interference 20&deg; minimum, satellite 45&deg; altitude
* Position geometry needs an `angle` method
* Angle(a, b) returns the angle from a through the origin to b
* Angle(a, b, o) returns the angle from a through o to b 
* Cosine of angle through origin is dot product of a and b
* Used in beam angle calculations

---

#### [can_see conic](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/position.rs#L68-L89)

* Initial "Can See" implementation
* Hung up on "angle above the horizon"
* Calculates whether the satellite is within a cone originating from the ground
* First projects satellite onto normal through ground
* Checks if above or below the horizon at all (t > 0)
* Checks if distance from satellite to normal is less than radius of cone at height
* Has a failing test satellite?
* Debugging regression test has `acos<f32>(1.000065)` returning NaN
* Special case for r~=1

---

#### can_see angle

* "Decompiled" the evaluator script
* Saw that it used the angle calculation
* Obviously it's not GOS
* Tried to get the angle calculation working on GSO, but it wasn't quite right
* Bolted awake at 5am realizing _tri_angles have _three_ corners
* Changed it SGO < 135&deg;, worked fine

---

## Algorithm, Implemented

---

### Satellite

#### [can_accept](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L90-L105)

* A satellite can accept a user if
  * < 32 users assigned
  * No self interference
  * No other interference

### [beam interference](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L62-L74)

* For all beams
  * assert angle from current user to satellite to attempted user is less than 10

--- 

### Satellite

#### [other interference](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L76-L88)

* For all interferers
  * assert angle from satellite to attempted user to interferer is less than 20

#### [find_best](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L46-L58)

* Actually just finds first which meets these criteria

---

### [optimize](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L28-L43)

* This algorithm needs to iterate over all band colors, all users, and all satellites.
* Choosing the order has implications for the algorithm's shape
* iterating users should be done against a mutable list, to remove users after
  they have been assigned.
  * If speed were a problem, removing full satellites might also be a good idea
  * Running quickly enough I didn't think that was worth the effort
* Should we go by user then satellite, or satellite then user?
  * Based on the decision for a `find_best`, it made sense to go user then satellite

---

### [optimize](https://github.com/DavidSouther/spacex_interview/blob/7fa7/src/solver.rs#L28-L43)
* Should we do band outermost or inner most?
  * Band is most constant, so it was intuitive to put it outside.
  * In practice, this means that the A band is chosen preferentially, the D band irregularly.
  * From the prompt, this is fine. Does this meet the reality of the satellites?
* Overall runtime is O(C N M), where C # 4 bands, N # users, M # satellites
  * I'm calling it O(N^2)

---

## First pass / it works

* Passes all checks
* Runs seconds, not minutes
* For simple tests, hits max coverage given eg cross pattern
* For common cases, >90% "felt good"
* For extreme cases, it worked
* 8 hours of development over two evenings
* Met all the requirements
* Send it in

---

## Second pass / weekend downtime

* 1440 satellites obviously cannot serve 100k users on 32 beams
  * 46,080 < 100,000
* 20% feels like it might actually be pretty good?
* Do other scenarios even have 100% coverage possible?
* Anyway, we need to get to a million users in the near future
* Let's write an analysis pass!

---

### Analysis questions

* How many satellites are completely saturated?
* How many satellites have no users assigned?
* How many users could this many satellites maximally serve?
* Given that number, what percentage of those were served?
* How many users were uncovered?
* Given the number of uncovered users, and the maximum satellite capacity, how good was the solution?

---

### Analysis not-questions

* Are there clusters of users within the network that overwhelm local capacity?
  * Does the example 3 five users or exmple 4 one interferer occur in the data set?
  * Does a cluster of, say, 49 (7x7 grid) of users fall underneath a single satellite?
* These would again reduce the available capacity

* [code](https://github.com/DavidSouther/spacex_interview/blob/main/src/main.rs#L35-L43)

---

### Results

| Test case | % Assigned | % Successful |
|-----------|------------|--------------|
| 08_eighteen_planes_northern.txt | 79.08% | 79.08% |
| 09_ten_thousand_users.txt | 91.94% | 97.75% |
| 10_ten_thousand_users_geo_belt.txt | 82.4% | 87.46% |
| 11_one_hundred_thousand_users.txt | 29.25% | 63.49% |

---

#### 08_eighteen_planes_northern.txt
        53 satellites are saturated
        254 satellites are unassigned
        A most, 2500 users can be served ( 100%)
                Current solution covered 1977 (79.079994%)
        0 users are completeley uncovered
        Solution is overall 79.079994% successful, given visiblity and capacity

#### 09_ten_thousand_users.txt
        101 satellites are saturated
        360 satellites are unassigned
        A most, 10000 users can be served ( 100%)
                Current solution covered 9194 (91.939995%)
        594 users are completeley uncovered
        Solution is overall 97.746124% successful, given visiblity and capacity

---

#### 10_ten_thousand_users_geo_belt.txt
    104 satellites are saturated
    360 satellites are unassigned
    A most, 10000 users can be served ( 100%)
        Current solution covered 8240 (82.4%)
    579 users are completeley uncovered
    Solution is overall 87.46418% successful, given visiblity and capacity

#### 11_one_hundred_thousand_users.txt
    650 satellites are saturated
    11 satellites are unassigned
    A most, 46080 users can be served (46.079998%)
            Current solution covered 29251 (29.251%)
    26535 users are completeley uncovered
    Solution is overall 63.478733% successful, given visiblity and capacity

---

## Improve the algorithm 

### Improve find_best

* By adding a heuristic, it may be possible to improve locallity of possible matches
* The more directly overhead the satellite, the better the beam (and lower chance of obstruction)
* Maybe find the next available satellite with the highest elevation?
* This approach does not appreciably improve the solution
  * Solution is better by ~2% points
  * Runtime increases by ~10%
  * Average elevation improved by 3 degrees
  * Simplifies geospatial positioning approaches in the future

---

### Geospatial Partitioning

* Adding a satellite should only add at most O(logN) to the runtime
* Choose best satellite by spatial partitioning
* Quadtree for most overhead? Uber H3?
* Threading per geospatioal region?

---

### spilling

* Using the metric above, when a ground station finds a very good satellite
  * the lower quality beam could be pushed to neighboring satellites
* This might result in that satellite spilling another connection
  * at worst case, it trades one "good" connection for one "bad" connection
* In a worst case, this becomes O(N*M*M), as we "spill" down a waterfal
  * In practice, this would more likely be O(NMrtM)
* I did not implement this approach

---

## Improving the network

### Baseline

* 1440 satellites with 32 beams at 10&deg; separation

```
Assigned 29243 users
Analysis: 
    649 satellites are saturated
    0 satellites are unassigned
    A most, 46080 users can be served (46.079998%)
        Current solution covered 29243 (29.243002%)
    26535 users are completeley uncovered
    Solution is overall 63.461376% successful, given visiblity and capacity
    Average dishy elevation is 59.14712 deg
```

---

### More beams

* Increase from 32 to 64 Beams roughly doubles capacity:

```
Assigned 46049 users
Analysis: 
    1 satellites are saturated
    0 satellites are unassigned
    A most, 92160 users can be served (92.16%)
        Current solution covered 46049 (46.05%)
    26535 users are completeley uncovered
    Solution is overall 57.78% successful, given visiblity and capacity
    Average dishy elevation is 59.11 deg
```

---

### Tighter Beams

* Decrease from 10* to 5* :

```
Analysis: 
    669 satellites are saturated
    0 satellites are unassigned
    A most, 46080 users can be served (46.08%)
        Current solution covered 29559 (29.56%)
    26535 users are completeley uncovered
    Solution is overall 64.15% successful, given visiblity and capacity
    Average dishy elevation is 59.91 deg
```

---

### Timesliced Beams

* By reducing bandwidth, beams can time slice for nearby users.
* Each color gets divided into ??? bands (I don't know the bandwidth of a channel)
* Most of the time, most users aren't both talking, so can preferentially give users more time when others are quiet

---

### More Tighter Beams

* 64 5&deg; beams:
```
Assigned 47094 users
Analysis: 
    3 satellites are saturated
    0 satellites are unassigned
    A most, 92160 users can be served (92.16%)
        Current solution covered 47094 (47.09%)
    26535 users are completeley uncovered
    Solution is overall 59.08% successful, given visiblity and capacity
    Average dishy elevation is 59.53 deg
```

---

### More satellites

* Fill in shells from FTC filing
  * On the one hand, you guys have spent way more time thinking about this than I have
  * On the other, this is basically the same principles I put together in KSP
* 1440 to 2880:
  * 1440 satellites in 72 orbits, evenly spaced along the orbit, orbits with 5&deg; increasing LAN, at 540km, 53.2&deg; inclination

* 2880 to 3600
  * 720 in a 570 km (350 mi) shell at 70º inclination (40 / orbit, 36 orbits)
  * Fourth shell: 336 in a 560 km (350 mi) shell at 97.6º (56 / orbit, 6 orbits)
  * Fifth shell: 172 satellites in a 560 km (350 mi) shell at 97.6º (43 / orbit, 4 orbits)

---

### Add second shell

    Read scenario
        100000 users
        2880 sats
        36 interferrers
    Assigned 43238 users
    Analysis: 
        1006 satellites are saturated
        589 satellites are unassigned
        A most, 92160 users can be served (92.16%)
                Current solution covered 43238 (43.24%)
        18198 users are completeley uncovered
        Solution is overall 48.71% successful, given visiblity and capacity
        Average dishy elevation is 59.69 deg

    real    2m33.415s
    user    2m32.892s
    sys     0m0.209s

--- 

### Getting to 1 million

* Single user per beam is biggest current drawback
* Adding more beams helps, adding tighter beams helps
* Adding satellites does not have issues with cross interference
* Adding satellites improves geographic coverage

---

## Telescopes?

* There's a big flat area under the solar panel on the backside of the satellite
* The satellites are travelling within the GPS shells
* 4k scientific CCDs are available commodity off the shelf
* Operating temperatures are uncertain; how much heat does a satellite generate?
* Putting a 4k camera with an appropriate low-profile lense could inexpensively turn the satellites into the largest synoptic survey available.