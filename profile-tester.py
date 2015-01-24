from jsonschema import validate as validate_schema
import json
from time import strptime
from hashlib import sha256
from passlib.utils.pbkdf2 import pbkdf2
from Crypto.Cipher import AES

STATUSES = ["", "Started", "Urgent"]
DATEFMT = "%Y-%m-%d %H:%M:%S %z"
PASSPHRASE = "DEBUG"

def decrypt_profile(ciphertext, passphrase):
  key = pbkdf2(bytes(passphrase.encode("utf-8")), sha256(b"DEBUG").hexdigest().encode("utf-8"), 2056, 32, "hmac-sha256")
  iv = ciphertext[0:16]
  decryptor = AES.new(key, AES.MODE_CBC, iv)
  plaintext = decryptor.decrypt(ciphertext[16:])
  try:
    return plaintext[:-plaintext[-1]].decode("utf-8")
  except UnicodeDecodeError:
    raise AssertionError("profile could not be decrypted")

def read_enc_json_file(path, pp):
  with open(path, "rb") as f:
    data = f.read()
  try:
    return json.loads(decrypt_profile(data, pp))
  except ValueError:
    raise AssertionError("profile contains invalid json")

def read_json_file(path):
  a = open(path)
  try:
    return json.load(a)
  except ValueError:
    raise AssertionError("profile contains invalid json")

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
c = read_enc_json_file("/home/roland/.theca/enc4.json", PASSPHRASE)

validate_schema(b, s) # heuheuheuheuh
validate_profile(b)

validate_schema(c, s)
validate_profile(c)
