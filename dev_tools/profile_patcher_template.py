#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# license under the MIT license <http://opensource.org/licenses/MIT>
#
# profile_patcher_template.py
#   framework template for patching theca profile JSON files.

import argparse, os, sys, json

PATCHER_VERSION = "0.1-dev"

##################
# PATCH FUNCTION #
##################

# this is where you should write the patching function, it will be 
# passed the path for what is (probably) a profile, a variable verbose,
# indicating if the function should be verbose, and a variable containing
# either None or a bool indicating the user answer to a prompt. if the
# profile is encrpyted then `decrypt_profile` should be used. this function
# should reutrn true, meaning sucess, or false, meaning failure so we can
# exit asap. all error reporting should be done inside PATCH_FUNCTION.
def PATCH_FUNCTION(profile_path, verbose, prompt_answer):
	import random
	if verbose:
		sys.stdout.write("IM GOING TO PATCH "+profile_path+"\n")
	return bool(random.randrange(0,2))

#####################
# PATCH INFORMATION #
#####################

# this is where we provide a short name for a patch (indicating version change, 
# format change, etc) as well as a longer description that provides some
# information about what the patch is going to do to a profile. everything
# should be mandatory...
PATCH_TITLE = "theca skeleton profile patching framework template"
PATCH_AUTHOR = "roland bracewell shoemaker <rolandshoemaker@gmail.com>"
PATCH_DESC_LONG = """this is a simple skeleton template for writing patches
to convert/fix/update theca profile files."""
PATCH_DATE = "25/01/2015"
PATCH_FILENAME = os.path.basename(__file__)

#################
# PATCHER UTILS #
#################

# a courtesy function that *properly* derives a 256-bit AES key using
# `passphrase` and decrypts a supplied ciphertext (as bytes) with the
# derived key, the plain text can then be loaded as any other JSON
# string (json.loads()).
def decrypt_profile(ciphertext, passphrase):
  from hashlib import sha256
  from passlib.utils.pbkdf2 import pbkdf2
  from Crypto.Cipher import AES
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

# read a encrypted profile and decrypt it using passphare `pp` and
# return a JSON object.
def read_enc_json_file(path, pp):
  with open(path, "rb") as f:
    data = f.read()
  try:
    return json.loads(decrypt_profile(data, pp))
  except ValueError:
    raise AssertionError("profile contains invalid json")

# read a plain text profile and return a JSON object.
def read_json_file(path):
  a = open(path)
  try:
    return json.load(a)
  except ValueError:
    raise AssertionError("profile contains invalid json")

# get yes or no with a pretty prompt, thx stackoverflow
# (http://stackoverflow.com/a/3041990).
def query_yes_no(question, default="yes"):
    valid = {"yes": True, "y": True, "ye": True,
             "no": False, "n": False}
    if default is None:
        prompt = " [y/n] "
    elif default == "yes":
        prompt = " [Y/n] "
    elif default == "no":
        prompt = " [y/N] "
    else:
        raise ValueError("invalid default answer: '%s'" % default)

    while True:
        sys.stdout.write(question + prompt)
        choice = input().lower()
        if default is not None and choice == '':
            return valid[default]
        elif choice in valid:
            return valid[choice]
        else:
            sys.stdout.write("Please respond with 'yes' or 'no' "
                             "(or 'y' or 'n').\n")

# return a list of paths of json files in `folder` or in the '.theca'
# folder in the users home directory.
def get_profiles(folder): # =None):
	if folder == None:
		folder = os.path.join(os.path.expanduser("~"), ".theca")
	if os.path.isdir(folder):
		paths = []
		for p in os.listdir(folder):
			if p.endswith(".json"):
				paths.append(os.path.join(folder, p))
		return paths
	else:
		sys.stdout.write(folder+" is not a folder.\n")

################
# PATCHER MAIN #
################

# failure+result wrapper
def patch_wrapper(path, ignore_fail, verbose, prompt_answer):
	if not PATCH_FUNCTION(path, verbose, prompt_answer):
		sys.stdout.write("[FAILED] failed applying patch to "+path+"\n")
		if not ignore_fail:
			exit(1)
	else:
		sys.stdout.write("[PATCHED] "+path+"\n")

# 
def add_globals(parser):
	parser.add_argument(
		"-f",
		"--profile-folder",
		help="where to look for theca profile files"
	)
	parser.add_argument(
		"--verbose",
		action="store_true",
		help="be rather verbose"
	)
	parser.add_argument(
		"--yes",
		action="store_true",
		help="answer yes to any prompts"
	)
	parser.add_argument(
		"--no",
		action="store_true",
		help="answer no to any prompts"
	)

# somewhat of a weird argparse workaround... need a better way to
# specify global arguments (eg. --profile-folder) and stuff like that
class Patcher(object):
	# patcher class, --profile-folder really should be able to be here... :<
	def __init__(self):
		parser = argparse.ArgumentParser(
			usage=PATCH_FILENAME+""" <command> [<args>]

"""+"[theca profile patcher "+PATCHER_VERSION+"] "+PATCH_TITLE+"""

commands:
  patch <profile>                  patch a single profile.
  patch_all                        patch all profiles in the profile folder.
  patch_details                    print details about the patch.

options:
  -f PATH, --profile-folder PATH   set the folder from which to read profile files
                                   [defaults to ~/.theca].
  --ignore-fail                    ignore patch failure when using subcommand
                                   patch_all instead of exiting.
  --verbose                        be rather verbose.
  --yes                            answer yes to any prompts.
  --no                             answer no to any prompts.
"""
		)
		parser.add_argument("command", help="command to run")
		args = parser.parse_args(sys.argv[1:2])
		if not hasattr(self, args.command):
			sys.stdout.write("unrecognized command\n")
			parser.print_help()
			exit(1)
		getattr(self, args.command)()

	# patch a single profile in the profiles folder using PATCH_FUNCTION
	def patch(self):
		parser = argparse.ArgumentParser(description="patch a single profile")
		add_globals(parser)
		# parser.add_argument(
		# 	"-f",
		# 	"--profile-folder",
		# 	help="where to look for theca profile files"
		# )
		# parser.add_argument(
		# 	"--verbose",
		# 	action="store_true",
		# 	help="be rather verbose"
		# )
		parser.add_argument('profile')
		args = parser.parse_args(sys.argv[2:])
		profile_folder = args.profile_folder or \
			os.path.join(os.path.expanduser("~"), ".theca")
		profile_path = os.path.join(profile_folder, args.profile+".json")
		if os.path.isfile(profile_path):
			if args.yes or args.no:
				prompt_answer = (args.yes or not args.no)
			else:
				prompt_answer = None
			patch_wrapper(profile_path, None, args.verbose, prompt_answer)
		else:
			sys.stdout.write(profile_path+" doesn't exist, so i can't patch it!\n")

	# patch all profiles in the profiles folder using PATCH_FUNCTION
	def patch_all(self):
		parser = argparse.ArgumentParser(description="patch all profiles")
		add_globals(parser)
		parser.add_argument(
			"--ignore-fail",
			action="store_true",
			help="don't exit if a patch fails")
		args = parser.parse_args(sys.argv[2:])
		if args.yes or args.no:
			prompt_answer = (args.yes or not args.no)
		else:
			prompt_answer = None
		if prompt_answer or \
		  query_yes_no("are you sure you want to patch all profiles?", default="no"):
			for p in get_profiles(args.profile_folder):
				patch_wrapper(p, args.ignore_fail, args.verbose, prompt_answer)

	# print all the details about the patch
	def patch_details(self):
		sys.stdout.write(
			"[theca profile patcher {}] {} - {}\n\n".format(
				PATCHER_VERSION,
				PATCH_TITLE,
				PATCH_DATE
			)
		)
		sys.stdout.write("author\n------\n"+PATCH_AUTHOR+"\n\n")
		sys.stdout.write("description\n-----------\n"+PATCH_DESC_LONG+"\n\n")

###########
# RUNTIME #
###########

if __name__ == "__main__":
	Patcher()
