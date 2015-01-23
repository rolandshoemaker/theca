from jsonschema import validate as validate_schema
import json
from time import strptime

STATUSES = ["", "Started", "Urgent"]
DATEFMT = "%Y-%m-%d %H:%M:%S %z"

def read_json_file(path):
  a = open(path)
  return json.load(a)

def validate_profile(profile):
  for i, n in enumerate(profile['notes']):
    if i > 0 and i < len(profile['notes'])-1:
      if not profile['notes'][i-1]['id'] < n['id'] < profile['notes'][i+1]['id']:
        raise AssertionError("object #"+str(i)+" id is out of order ("+str(profile['notes'][i-1]['id'])+", "+str(n['id'])+", "+str(profile['notes'][i+1]['id'])+")")
      if n['status'] not in STATUSES: raise AssertionError("")
    if n['id'] < 0: raise AssertionError("object #"+str(i)+" id is negative ("+str(n['id'])+")")
    try:
      strptime(n['last_touched'], DATEFMT)
    except ValueError:
      raise AssertionError("object #"+str(i)+" last_touched doesn't match time format %Y-%m-%d %H:%M:%S %z")
    profile_ids = [n['id'] for n in profile['notes']]
    if len(profile_ids) != len(set(profile_ids)): raise AssertionError("there are duplicate IDs in 'notes'")

s = read_json_file("schema.json")
b = read_json_file("/home/roland/.theca/default.json")

validate_schema(b, s) # heuheuheuheuh
validate_profile(b)
