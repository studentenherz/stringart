from math import sqrt

with open("lines.txt", "r") as file:
    length = 0
    for line in file.readlines():
        [x1, y1, x2, y2] = [int(x) for x in line.split()]
        length += sqrt((x1 - x2) ** 2 + (y1 - y2) ** 2)

    print(f"Total length = {length}")
