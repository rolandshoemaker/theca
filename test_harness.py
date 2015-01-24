from jsonschema import validate as validate_schema
import json
from time import strptime
from hashlib import sha256
from passlib.utils.pbkdf2 import pbkdf2
from Crypto.Cipher import AES
import tempfile
import os.path
from shutil import rmtree
import subprocess
import os

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

def validate_profile_schema(profile):
  validate_schema(profile, SCHEMA)

def validate_profile_contents(profile):
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

def compare_profile(clean, dirty):
  if not clean['encrypted'] == dirty['encrypted']: raise AssertionError()
  if not len(clean['notes']) == len(dirty['notes']): raise AssertionError()
  for c, d in zip(clean['notes'], dirty['notes']):
    if not c['id'] == d['id']: raise AssertionError()
    if not c['title'] == d['title']: raise AssertionError()
    if not c['status'] == d['status']: raise AssertionError()
    if not c['body'] == d['body']: raise AssertionError()
    # uh leaving last_touched for now...

def test_harness(tests):
  TMPDIR = tempfile.mkdtemp()
  devnull = open(os.devnull, "w")
  failed = 0

  print("# running {} tests.".format(len(tests)))
  for t in tests:
    try:
      print("test: "+t['name'])
      cmd = [THECA_CMD]
      if not t["profile"] == "":
        cmd += ["-p", t["profile"]]
      if not t["profile_folder"] == "":
        cmd += ["-f", os.path.join(TMPDIR, t["profile_folder"])]
      else:
        cmd += ["-f", TMPDIR]
      
      if len(t["stdin"]) > 0:
        for c, s in zip(t["cmds"], t["stdin"]):
          if not s == None:
            p = subprocess.Popen(c, stdin=subprocess.PIPE, stdout=devnull)
            p.communicate(input=bytes(s))
          else:
            subprocess.call(cmd+c, stdout=devnull)
      else:
        for c in t["cmds"]:
          subprocess.call(cmd+c, stdout=devnull)

      result_path = os.path.join(TMPDIR, t["result_path"])
      if t["result"]["encrypted"]:
        json_result = read_enc_json_file(result_path, t["result_passphrase"])
      else:
        json_result = read_json_file(result_path)
      validate_profile_schema(json_result)
      validate_profile_contents(json_result)
      compare_profile(t["result"], json_result)
    except AssertionError:
      failed += 1

    os.remove(result_path)

  rmtree(TMPDIR)
  devnull.close()

  print("tests passed: {}, failed {}.".format(len(tests)-failed, failed))
  if failed > 0:
    exit(1)


TESTS = [
  {
    "name": "new profile",
    "profile": "",
    "profile_folder": "",
    "cmds": [
      ["new-profile"]
    ],
    "stdin": [],
    "result_path": "default.json",
    "result_passphrase": "",
    "result": {
      "encrypted": False,
      "notes": []
    }
  },{
    "name": "add note",
    "profile": "",
    "profile_folder": "",
    "cmds": [
      ["new-profile"],
      ["add", "this is the title"]
    ],
    "stdin": [],
    "result_path": "default.json",
    "result_passphrase": "",
    "result": {
      "encrypted": False,
      "notes": [
        {
          "id": 1,
          "title": "this is the title",
          "status": "",
          "body": ""
        }
      ]
    }
  }
]

THECA_CMD = "target/theca"

STATUSES = ["", "Started", "Urgent"]
DATEFMT = "%Y-%m-%d %H:%M:%S %z"
PASSPHRASE = "DEBUG"
SCHEMA = read_json_file("schema.json")

test_harness(TESTS)

# b = read_json_file("/home/roland/.theca/default.json")
# c = read_enc_json_file("/home/roland/.theca/enc4.json", PASSPHRASE)

# validate_profile_schema(b) # heuheuheuheuh
# validate_profile_contents(b)

# validate_profile_schema(c)
# validate_profile_contents(c)
