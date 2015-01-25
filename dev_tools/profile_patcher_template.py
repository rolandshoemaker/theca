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

import argparse, os, sys

PATCHER_VERSION = "0.1-dev"

##################
# PATCH FUNCTION #
##################

# this is where you should write the patching function, it will be 
# passed the path for what is (probably) a profile, some kind of check
# should probably be made to make sure random json files don't get patched.
def PATCH_FUNCTION(profile_path):
	sys.stdout.write("IM GOING TO PATCH "+profile_path+"\n")
	return 0

#####################
# PATCH INFORMATION #
#####################

# this is where we provide a short name for a patch (indicating version change, 
# format change, etc) as well as a longer description that provides some
# information about what the patch is going to do to a profile. everything
# should be mandatory...
PATCH_TITLE = "theca skeleton profile patching framework template"
PATCH_AUTHOR = "roland bracewell shoemaker"
PATCH_DESC_LONG = "this is a simple skeleton template for writing patches to convert/fix/update theca profile files."
PATCH_DATE = "25/01/2015"
PATCH_FILENAME = "profile_patcher_template.py"

#################
# PATCHER UTILS #
#################

# a courtesy function that *properly* derives a 256-bit AES key using
# `passphrase` and decrypts a supplied ciphertext (as bytes) with the
# derived key, the plain text can then be loaded as any other JSON
# string (json.loads())
def decrypt_profile(ciphertext, passphrase):
  from hashlib import sha256
  from passlib.utils.pbkdf2 import pbkdf2
  from Crypto.Cipher import AES
  key = pbkdf2(bytes(passphrase.encode("utf-8")), sha256(b"DEBUG").hexdigest().encode("utf-8"), 2056, 32, "hmac-sha256")
  iv = ciphertext[0:16]
  decryptor = AES.new(key, AES.MODE_CBC, iv)
  plaintext = decryptor.decrypt(ciphertext[16:])
  try:
    return plaintext[:-plaintext[-1]].decode("utf-8")
  except UnicodeDecodeError:
    raise AssertionError("profile could not be decrypted")

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
		parser.add_argument("-f", "--profile-folder", help="where to look for theca profile files")
		parser.add_argument('profile')
		args = parser.parse_args(sys.argv[2:])
		profile_folder = args.profile_folder or os.path.join(os.path.expanduser("~"), ".theca")
		profile_path = os.path.join(profile_folder, args.profile+".json")
		if os.path.isfile(profile_path):
			PATCH_FUNCTION(profile_path)
		else:
			sys.stdout.write(profile_path+" doesn't exist, so i can't patch it!\n")

	# patch all profiles in the profiles folder using PATCH_FUNCTION
	def patch_all(self):
		parser = argparse.ArgumentParser(description="patch all profiles")
		parser.add_argument("-f", "--profile-folder", help="where to look for theca profile files")
		args = parser.parse_args(sys.argv[2:])
		if query_yes_no("are you sure you want to patch all profiles?", default="no"):
			for p in get_profiles(args.profile_folder):
				PATCH_FUNCTION(p)

	# print all the details about the patch
	def patch_details(self):
		sys.stdout.write("[theca profile patcher "+PATCHER_VERSION+"] "+PATCH_TITLE+" - "+PATCH_DATE+"\n")
		sys.stdout.write("\tauthor: "+PATCH_AUTHOR+"\n")
		sys.stdout.write("\tdescription: "+PATCH_DESC_LONG+"\n")

###########
# RUNTIME #
###########

if __name__ == "__main__":
	Patcher()
