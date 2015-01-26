#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# license under the MIT license <http://opensource.org/licenses/MIT>
#
# theca_test_harness.py
#   external python testing harness for testing the theca cli binary.

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
import time

def decrypt_profile(ciphertext, passphrase):
  key = pbkdf2(bytes(passphrase.encode("utf-8")), sha256(bytes(passphrase.encode("utf-8"))).hexdigest().encode("utf-8"), 2056, 32, "hmac-sha256")
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

def validate_profile_schema(profile, schema):
  validate_schema(profile, schema)

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
      print(n)
      raise AssertionError("object #"+str(i)+" last_touched doesn't match time format "+DATEFMT)
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
  SCHEMA = read_json_file(SCHEMA_PATH)

  print("# {}\n#    {}".format(tests['title'], tests['desc']))
  print("#\n# running {} tests.\n".format(len(tests['tests'])))
  for t in tests['tests']:
    try:
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
            p = subprocess.Popen(cmd+c, stdin=subprocess.PIPE, stdout=devnull)
            p.communicate(input=bytes(s.encode('utf-8')))
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
      validate_profile_schema(json_result, SCHEMA)
      validate_profile_contents(json_result)
      compare_profile(t["result"], json_result)
      print("\ttest: "+t['name']+" [PASSED]")
    except (AssertionError, FileNotFoundError) as e:
      print("\033[91m"+"\ttest: "+t['name']+" [FAILED]"+"\033[0m")
      failed += 1

    # os.remove(result_path)
    for f_o in os.listdir(TMPDIR):
      f_o_p = os.path.join(TMPDIR, f_o)
      if os.path.isfile(f_o_p):
        os.unlink(f_o_p)
      else:
        shutil.rmtree(f_o_p)

  rmtree(TMPDIR)
  devnull.close()
  print("\n[passed: {}, failed {}]\n".format(len(tests['tests'])-failed, failed))
  return failed

# GOOD_TESTS = read_json_file("tests/good_tests.json")
# BAD_TESTS = read_json_file("tests/bad_tests.json")

ALL_TESTS = [
  "tests/good_default_tests.json",
  "tests/good_second_profile_tests.json",
  "tests/good_encrypted_profile_tests.json",
  "tests/bad_tests.json"
]

THECA_CMD = "theca"

STATUSES = ["", "Started", "Urgent"]
DATEFMT = "%Y-%m-%d %H:%M:%S %z"
SCHEMA_PATH = "schema.json"

if __name__ == "__main__":
  test_sum = 0
  failed = 0
  start = time.time()
  for t_set_path in ALL_TESTS:
    t_set = read_json_file(t_set_path)
    test_sum += len(t_set['tests'])
    failed += test_harness(t_set)
  elapsed = time.time()-start
  m, s = divmod(elapsed, 60)
  h, m = divmod(m, 60)

  print("ran %d tests overall: %d passed, %d failed, took %02d:%02d:%02d\n" % (test_sum, test_sum-failed, failed, h, m, s))

  if failed > 0:
    print("\033[91m"+"BAD"+"\033[0m")
    exit(1)
