#!/usr/bin/python3

from random import normalvariate
from random import expovariate
import sys

for i in range(0, 100000):
    if (sys.argv[1] == "expo"):
        print(expovariate(1 / 500))
    elif (sys.argv[1] == "normal"):
        print(normalvariate(300, 50))

