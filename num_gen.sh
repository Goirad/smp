#!/usr/bin/python3

from random import normalvariate;
from random import expovariate;

for i in range(0, 100000):
	print(expovariate(1 / 500))
