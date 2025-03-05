#!/usr/bin/python3

import json

last = None

def thing(x, i):
    global last
    for (k, v) in x.items():
        if isinstance(v, dict):
            if last is not None and last != dict:
                print()
            last = dict
            print("#"*i, k.replace("_", " ").title())
            print()
            thing(v, i+1)
        elif isinstance(v, str):
            print(f"- `{k}`: {v}")
            last = str
        elif isinstance(v, list):
            for line in v:
                print(line)
            last = list

with open("default-config.json", "r") as f:
    x = json.loads(f.read())["docs"]
    del x["description"]
    thing(x, 4)
