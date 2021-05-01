#!/usr/bin/env python3.7
from math import sqrt, acos, degrees
from collections import namedtuple
from sys import argv, stdin

Vector = namedtuple('Vector3', ['x', 'y', 'z'])
Origin = Vector(0, 0, 0)
beams = [str(A+1) for A in range(0, 32)]
bands = [chr(ord('A')+A)for A in range(0, 4)]
visible_elevation = 45.0
band_separation = 10.0
interference_separation = 20.0


def sub(A, B):
    return Vector(A.x-B.x, A.y-B.y, A.z-B.z)


def norm(A):
    # ||N|| = N/|N|
    n = sqrt(A.x**2+A.y**2+A.z**2)
    return Vector(A.x/n, A.y/n, A.z/n)


def dot(A, B):
    return A.x*B.x+A.y*B.y+A.z*B.z


def angle(O, A, B):
    J = norm(sub(A, O))
    K = norm(sub(B, O))
    L = dot(J, K)

    M = min(1.0, max(-1.0, L))
    if abs(M-L) > 1e-06:
        print(f"dot_product: {L} bounded to {M}")
    return degrees(acos(M))


def check_beams(scenario, solution):
    solution = solution
    scenario = scenario
    print('Checking no sat interferes with itself...')
    for satellite in solution:
        beam_assignments = solution[satellite]
        E = list(beam_assignments.keys())
        Q = scenario["sats"][satellite]
        for G in range(len(beam_assignments)):
            for N in range(G+1, len(beam_assignments)):
                R = beam_assignments[E[G]][1]
                S = beam_assignments[E[N]][1]
                if R != S:
                    continue
                T = beam_assignments[E[G]][0]
                U = beam_assignments[E[N]][0]
                V = scenario["users"][T]
                W = scenario["users"][U]
                P = angle(Q, V, W)
                if P < band_separation:
                    print(
                        f"\tSat {satellite} beams {E[G]} and {E[N]} interfere.")
                    print(f"\t\tBeam angle: {P} degrees.")
                    return False
    print('\tNo satellite self-interferes.')
    return True


def check_interferers(scenario, solution):
    print('Checking no sat interferes with a non-Starlink satellite...')
    for satelite in solution:
        N = scenario["sats"][satelite]
        for beam in solution[satelite]:
            O = solution[satelite][beam][0]
            P = scenario["users"][O]
            for K in scenario["interferers"]:
                Q = scenario["interferers"][K]
                M = angle(P, N, Q)
                if M < interference_separation:
                    print(
                        f"\tSat {satelite} beam {beam} interferes with non-Starlink sat {K}.")
                    print(f"\t\tAngle of separation: {M} degrees.")
                    return False
    print('\tNo satellite interferes with a non-Starlink satellite!')
    return True


def check_coverage(scenario, solution):
    C = solution
    print('Checking user coverage...')
    E = []
    for I in solution:
        for K in C[I]:
            G = C[I][K][0]
            if G in E:
                print(f"\tUser {G} is covered multiple times by solution!")
                return False
            E.append(G)
    J = len(scenario["users"])
    L = len(E)
    print(f"{L/J*100}% of {J} total users covered.")
    return True


def check_visibility(scenario, solution):
    scenario = scenario
    solution = solution
    print('Checking each user can see their assigned satellite...')
    for satelite in solution:
        for beam in solution[satelite]:
            user = solution[satelite][beam][0]
            M = scenario["users"][user]
            N = scenario["sats"][satelite]
            K = angle(M, Origin, N)
            if K <= 180.0-visible_elevation:
                P = str(K-90)
                print(
                    f"\tSat {satelite} outside of user {user}'s field of view.")
                print(f"\t\t{P} degrees elevation.")
                print(f"\t\t(Min: {90-visible_elevation} degrees elevation.)")
                return False
    print("\tAll users' assigned satellites are visible.")
    return True


def parse_line(object_type, line, dest):
    tokens = line.split()
    if tokens[0] != object_type or len(tokens) != 5:
        print("Invalid line! " + line)
        return False
    else:
        entity = tokens[1]
        try:
            I = float(tokens[2])
            J = float(tokens[3])
            K = float(tokens[4])
        except:
            print("Can't parse location! "+line)
            return False
        dest[entity] = Vector(I, J, K)
        return True


def parse_scenario(filename, scenario):
    print('Reading scenario file ' + filename)
    lines = open(filename).readlines()
    scenario["sats"] = {}
    scenario["users"] = {}
    scenario["interferers"] = {}
    for line in lines:
        if '#' in line:
            continue
        elif line.strip() == '':
            continue
        elif 'interferer' in line:
            if not parse_line('interferer', line, scenario["interferers"]):
                return False
        elif 'sat' in line:
            if not parse_line('sat', line, scenario["sats"]):
                return False
        elif 'user' in line:
            if not parse_line('user', line, scenario["users"]):
                return False
        else:
            print("Invalid line! " + line)
            return False
    return True


def parse_solution(filename, scenario, solution):
    if filename == '':
        print('Reading solution from stdin.')
        file = stdin
    else:
        print(f"Reading solution file {filename}.")
        file = open(filename)
    lines = file.readlines()
    for line in lines:
        tokens = line.split()
        if '#' in line:
            continue
        elif len(tokens) == 0:
            continue
        elif len(tokens) == 8:
            if tokens[0] != 'sat' or tokens[2] != 'beam' or tokens[4] != 'user' or tokens[6] != 'color':
                print("Invalid line! " + line)
                return False
            sat = tokens[1]
            beam = tokens[3]
            user = tokens[5]
            color = tokens[7]
            if not sat in scenario["sats"]:
                print('Referenced an invalid sat id! ' + line)
                return False
            if not user in scenario["users"]:
                print('Referenced an invalid user id! ' + line)
                return False
            if not beam in beams:
                print('Referenced an invalid beam id! ' + line)
                return False
            if not color in bands:
                print('Referenced an invalid color! ' + line)
                return False
            if not sat in solution:
                solution[sat] = {}
            if beam in solution[sat]:
                print('Beam is allocated multiple times! ' + line)
                return False
            solution[sat][beam] = user, color
        else:
            print("Invalid line! " + line)
            return False
    file.close()
    return True


def i():
    if len(argv) != 3 and len(argv) != 2:
        print(
            'Usage: python3.7 evaluate.py /path/to/scenario.txt [/path/to/solution.txt]')
        print(
            '   If the optional /path/to/solution.txt is not provided, stdin will be read.')
        return -1
    scenario = {}
    if not parse_scenario(argv[1], scenario):
        return -1
    solution = {}
    if len(argv) != 3:
        if not parse_solution('', scenario, solution):
            return -1
    elif not parse_solution(argv[2], scenario, solution):
        return -1
    if not check_coverage(scenario, solution):
        return -1
    if not check_visibility(scenario, solution):
        return -1
    if not check_beams(scenario, solution):
        return -1
    if not check_interferers(scenario, solution):
        print(
            'Solution contained a beam that could interfere with a non-Starlink satellite.')
        return -1
    print('\nSolution passed all checks!\n')
    return 0


if __name__ == '__main__':
    exit(i())
