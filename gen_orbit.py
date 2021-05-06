#!/usr/bin/env python3.8
import argparse
from math import radians, sin, cos

parser = argparse.ArgumentParser(description='Process some integers.')
parser.add_argument('--id', type=int, default=1,
                    help="Starting ID for these satellites")
parser.add_argument('--satellites', type=int, required=True,
                    help='Number of satellites in this shell')
parser.add_argument('--planes', type=int, required=True,
                    help='Number of orbital planes to put satellites in')
parser.add_argument('--lan', type=float, required=True,
                    help='Longitude of Ascending Node for first orbit')
parser.add_argument('--inclination', type=float, required=True,
                    help='Inclination of orbital planes')
parser.add_argument('--altitude', type=float, required=True,
                    help='Altitude above surface, in km. Surface is at 6356')


def position(altitude, ascending, inclination, anomaly):
    # Periapsis = 0
    # Eccentricity = 0
    # Semimajor Axis = altitude
    p = 0

    i = radians(inclination)
    o = radians(ascending)
    w = p - o

    # Yay circular orbits
    E = radians(anomaly)
    xp = altitude * cos(E)
    yp = altitude * sin(E)

    # Decomposed rotation
    x = (cos(w) * cos(o) - sin(w) * sin(o) * cos(i)) * xp + \
        (-sin(w) * cos(o) - cos(w) * sin(o) * cos(i)) * yp
    y = (cos(w) * sin(o) + sin(w) * cos(o) * cos(i)) * xp + \
        (-sin(w) * sin(o) - cos(w) * cos(o) * cos(i)) * yp
    z = sin(w) * sin(i) * xp + cos(w) * sin(i) * yp

    return (x, y, z)


def gen_orbits(args):
    satellites_per_plane = args.satellites / args.planes
    if not satellites_per_plane.is_integer():
        raise Exception(
            f"Satellites per orbit must be a whole number ({args.planes} / {args.satellites} = {satellites_per_plane})")
    altitude = args.altitude + 6356
    planes = args.satellites / satellites_per_plane
    for plane in range(0, int(planes)):
        lan = 360 * plane / planes + args.lan
        for sat in range(0, int(satellites_per_plane)):
            anomaly = 360 * sat / satellites_per_plane
            yield position(altitude, lan, args.inclination, anomaly)


if __name__ == "__main__":
    args = parser.parse_args()
    id = args.id
    for (x, y, z) in gen_orbits(args):
        print(f"sat {id} {x} {y} {z}")
        id += 1
