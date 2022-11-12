#!/usr/bin/env python3

import toml
import json
import requests
from functools import reduce
import string
from os import path

TOML_FILE = "assets/planets/planets.toml"

def snake_case(str):
    return reduce(lambda x, y: x + ('_' if y.isupper() else '') + ('' if y in string.whitespace else y), str).lower()
    
def convert_body(body):
    # Change to snake case
    body = { snake_case(key): value for key, value in body.items() }
    
    body["identifier"] = snake_case(body["english_name"])
    body["name"] = body["english_name"]
    del body["english_name"]

    body["body_type"] = snake_case(body["body_type"])

    if body["identifier"] != "sun" and body["body_type"] in ["planet", "dwarf_planet"]:
        body["parent"] = "sun"

    body["volume"] = body["vol"]["volValue"] * (10 ** body["vol"]["volExponent"])
    del body["vol"]

    body["mass"] = body["mass"]["massValue"] * (10 ** body["mass"]["massExponent"])
    
    newBody = {
        "identifier": body["identifier"],
        "name": body["name"],
    }

    for key, value in body.items():
        if key not in newBody:
            newBody[key] = value
    
    return newBody
        

if __name__ == "__main__":
    x = requests.get("https://api.le-systeme-solaire.net/rest/bodies/?filter[]=bodyType,eq,Dwarf%20planet&filter[]=bodyType,eq,Planet&filter[]=bodyType,eq,Star&satisfy=any&data=englishName,bodyType,semimajorAxis,periphelion,aphelion,eccentricity,inclination,mass,massValue,massExponent,vol,volValue,volExponent,density,gravity,meanRadius")
    
    bodies = [ convert_body(body) for body in x.json()["bodies"] ]
    bodies = { body["identifier"]: body for body in bodies }
    bodies_prev = []

    if path.exists(TOML_FILE):
        with open(TOML_FILE, "r") as f:
            bodies_prev = toml.load(f)["bodies"]
    else:
        print("WARNING: No planets.toml previously existed. Colours cannot be used")
        
    bodies_prev = { body["identifier"]: body for body in bodies_prev }
    
    def update_return(x, y):
        x.copy().update(y)
        return x
    
    for key, body in bodies.items():
        if key not in bodies_prev:
            bodies_prev[key] = body
            continue
            
        bodies_prev[key].update(body)
        
    bodies = list(bodies_prev.values())

    for body in bodies:
        if "colour" not in body and "name" in body:
            print(f"WARNING: {body['name']} has not been designated a colour")
        
        if "parent" not in body and "name" in body:
            print(f"WARNING: {body['name']} has no parent")
    
    with open(TOML_FILE, "w") as f:
        toml.dump({ "bodies": bodies }, f)
        
    
        
