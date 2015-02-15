#!/usr/bin/python3
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# theca_test_harness.py - v0.9.0
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
import os, sys
import time
import argparse

PROFILE_TESTS = [
    "tests/good_default_tests.json",
    "tests/good_second_profile_tests.json",
    "tests/good_encrypted_profile_tests.json",
    "tests/bad_input_good_profile_tests.json"
]

JSON_OUTPUT_TESTS = [
    "tests/good_json_list_tests.json",
    "tests/good_json_search_tests.json"
]

TEXT_OUTPUT_TESTS = [
    "tests/good_text_output_tests.json"
]

THECA_CMD = "theca"

STATUSES = ["", "Started", "Urgent"]
DATEFMT = "%Y-%m-%d %H:%M:%S %z"
SCHEMA_PATH = "docs/schema.json"

def decrypt_profile(ciphertext, passphrase):
    key = pbkdf2(
        bytes(passphrase.encode("utf-8")),
        sha256(bytes(passphrase.encode("utf-8"))).hexdigest().encode("utf-8"),
        2056,
        32,
        "hmac-sha256"
    )
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
                raise AssertionError(
                    "object #%d id is out of order (%d, %d, %d)" %
                    (i,
                    profile['notes'][i-1]['id'],
                    n['id'])
                )
            if n['status'] not in STATUSES: raise AssertionError()
        if n['id'] < 0:
            raise AssertionError(
                "object #%d id is negative (%d)" % 
                (i,
                n['id'])
            )
        try:
            strptime(n['last_touched'], DATEFMT)
        except ValueError:
            raise AssertionError(
                "object #%d last_touched doesn't match time format %s" % 
                (i,
                DATEFMT)
            )
        profile_ids = [n['id'] for n in profile['notes']]
        if len(profile_ids) != len(set(profile_ids)):
            raise AssertionError("there are duplicate IDs in 'notes'")

def compare_notes(clean, dirty):
    try:
        if not clean['id'] == dirty['id']: raise AssertionError()
        if not clean['title'] == dirty['title']: raise AssertionError()
        if not clean['status'] == dirty['status']: raise AssertionError()
        if not clean['body'] == dirty['body']: raise AssertionError()
        # uh leaving last_touched for now...
    except AssertionError:
        raise AssertionError("# EXPECTED #\n"+json.dumps(clean, indent=2)+"\n# GOT #\n"+json.dumps(dirty, indent=2))

def compare_profile(clean, dirty):
    if not clean['encrypted'] == dirty['encrypted']: raise AssertionError()
    if not len(clean['notes']) == len(dirty['notes']): raise AssertionError()
    for c, d in zip(clean['notes'], dirty['notes']):
        compare_notes(c, d)

def run_cmds(
    cmds,
    profile,
    profile_folder,
    tmpdir,
    stdin=[],
    get_output=False,
    wait=None,
    should_fail=False
):
    if get_output:
        stdout = subprocess.PIPE
    else:
        stdout = open(os.devnull, "w")
    cmd = [THECA_CMD]
    if not profile == None:
        cmd += ["-p", profile]
    if not profile_folder == None:
        cmd += ["-f", os.path.join(tmpdir, profile_folder)]
    else:
        cmd += ["-f", tmpdir]

    if get_output:
        output = []
    if len(stdin) > 0:
        for c, s in zip(cmds, stdin):
            if wait: time.sleep(wait)
            if not s == None:
                stdin = subprocess.PIPE
                stdin_input = bytes(s.encode('utf-8'))
            else:
                stdin, stdin_input = None, None
            p = subprocess.Popen(cmd+c, stdin=stdin, stdout=stdout)
            if get_output:
                output += [p.communicate(input=stdin_input)[0].decode('utf-8')]
            else:
                p.communicate(input=stdin_input)
            if not p.returncode == 0 and not should_fail:
                raise AssertionError() 
    else:
        for c in cmds:
            if get_output:
                if wait: time.sleep(wait)
                output += [subprocess.Popen(cmd+c, stdout=stdout)
                                     .communicate()[0].decode('utf-8')]
            else:
                subprocess.Popen(cmd+c, stdout=stdout).communicate()
    if not stdout == subprocess.PIPE:
        stdout.close()
    if get_output:
        return output

def bench_harness(benches):
    pass

def test_harness(tests, cond=False):
    TMPDIR = tempfile.mkdtemp()
    SCHEMA = read_json_file(SCHEMA_PATH)
    failed = 0

    print("# {}\n#    {}".format(tests['title'], tests['desc']))
    if not cond:
        print("#\n# running {} tests.\n".format(len(tests['tests'])))
    for t in tests['tests']:
        if not cond:
            print("  test: "+t['name'], end="")
        sys.stdout.flush()
        try:
            if not t.get("result_type"):
                run_cmds(
                    t["cmds"],
                    t.get("profile", None),
                    t.get("profile_folder", None),
                    TMPDIR,
                    stdin=t.get("stdin", []),
                    should_fail=t.get("should_fail", False)
                )

                result_path = os.path.join(TMPDIR, t["result_path"])
                if t["result"]["encrypted"]:
                    json_result = read_enc_json_file(result_path, t["result_passphrase"])
                else:
                    json_result = read_json_file(result_path)
                validate_profile_schema(json_result, SCHEMA)
                validate_profile_contents(json_result)
                compare_profile(t["result"], json_result)
            else:
                results = run_cmds(
                    t["cmds"],
                    t.get("profile", None),
                    t.get("profile_folder", None),
                    TMPDIR,
                    get_output=True,
                    wait=t.get("cmd_interval", None),
                    should_fail=t.get("should_fail", False)
                )
                for clean, dirty in zip(t["results"], results):
                    if t['result_type'] == "json":
                        if type(clean) == list:
                            dirty = json.loads(dirty)
                            if len(clean) != len(dirty):
                                raise AssertionError("# EXPECTED #\n"+json.dumps(clean, indent=2)+"\n# GOT #\n"+json.dumps(dirty, indent=2))
                            for c, d in zip(clean, dirty):
                                if not c == None: compare_notes(c, d)
                        else:
                            if not clean == None: compare_notes(clean, json.loads(dirty))
                    elif t['result_type'] == "text":
                        if not clean == None and not clean == dirty:
                            raise AssertionError("# EXPECTED #\n"+json.dumps(clean, indent=2)+"\n# GOT #\n"+json.dumps(dirty, indent=2))
            if not cond:
                print(" [PASSED]")
            else:
                print(".", end="")
        except (AssertionError, FileNotFoundError) as e:
            if not cond:
                print("\033[91m"+" [FAILED]"+"\033[0m")
            else:
                print("\033[91mF\033[0m", end="")
            print(e)
            failed += 1
        for f_o in os.listdir(TMPDIR):
            f_o_p = os.path.join(TMPDIR, f_o)
            if os.path.isfile(f_o_p):
                os.unlink(f_o_p)
            else:
                shutil.rmtree(f_o_p)

    rmtree(TMPDIR)
    print("\n[passed: {}, failed {}]\n".format(len(tests['tests'])-failed, failed))
    return failed

if __name__ == "__main__":
    arg_parser = argparse.ArgumentParser(description="test harness for the theca cli binary.")
    arg_parser.add_argument("-tc", "--theca-command", help="where is the theca binary")
    arg_parser.add_argument("-tf", "--test-file", help="path to specific test file to run")
    arg_parser.add_argument(
        "-pt",
        "--profile-tests",
        action="store_true",
        help="only run the profile output tests"
    )
    arg_parser.add_argument(
        "-jt",
        "--json-tests",
        action="store_true",
        help="only run the json output tests"
    )
    arg_parser.add_argument(
        "-tt",
        "--text-tests",
        action="store_true",
        help="only run the text output tests"
    )
    arg_parser.add_argument("--condensed", action="store_true")
    args = arg_parser.parse_args()
    if args.theca_command:
        THECA_CMD = args.theca_command

    ALL_TESTS = []
    if args.profile_tests:
        ALL_TESTS += PROFILE_TESTS
    if args.json_tests:
        ALL_TESTS += JSON_OUTPUT_TESTS
    if args.text_tests:
        ALL_TESTS += TEXT_OUTPUT_TESTS
    if not args.text_tests and not args.json_tests and not args.profile_tests:
        if args.test_file:
            ALL_TESTS.append(args.test_file)
        else:
            ALL_TESTS += PROFILE_TESTS+JSON_OUTPUT_TESTS+TEXT_OUTPUT_TESTS

    test_sum = 0
    failed = 0
    start = time.time()

    for t_set_path in ALL_TESTS:
        t_set = read_json_file(t_set_path)
        test_sum += len(t_set['tests'])
        failed += test_harness(t_set, cond=args.condensed)

    elapsed = time.time()-start
    m, s = divmod(elapsed, 60)
    h, m = divmod(m, 60)

    print(
        "ran %d tests overall: %d passed, %d failed, took %02d:%02d:%02d\n" %
        (test_sum,
        test_sum-failed,
        failed,
        h,
        m,
        s)
    )

    if failed > 0:
        print("\033[91m"+"BAD"+"\033[0m")
        exit(1)
