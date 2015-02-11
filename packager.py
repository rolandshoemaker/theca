import platform, os, shutil, tempfile, tarfile
from subprocess import Popen, PIPE

# *Should* be run on clean git clone from the root of the repo...

def s_Popen(cmd, return_output=False):
	p = Popen(cmd, stdout=PIPE, stderr=PIPE)
	output = p.communicate()[0].decode("utf-8")
	if p.returncode > 0:
		# raise or something idk?
		pass
	if return_output:
		return output

OUTPUT_DIR = "/home/roland/testing"

RUST_CHANNEL = "nightly"
TARGET_ARCHS = ["x86_64", "i686"]
if platform.system() == "Linux":
	HOST_OS = "unknown-linux-gnu"
elif platform.system() == "Darwin":
	HOST_OS = "apple-darwin"
else:
    print("nop")
    exit(1)

PACKAGE_PREFIX = "theca-nightly" # from arg!!
PACKAGE_STATIC_CONTENT = {
	"install.sh": "install.sh",
	"README.md": "README.md",
	"LICENSE": "LICENSE",
	"docs/THECA.1": "share/man/man1/theca.1",
	"completion/bash_complete.sh": "etc/bash_completion.d/theca",
	"completion/_theca": "share/zsh/site-functions/_theca"
}

BUILD_CMD = ["cargo", "build", "--release"]

# setup tmpdir
TMPDIR = tempfile.mkdtemp()

# build static package structure 
package_dir = os.path.join(TMPDIR, PACKAGE_PREFIX)
os.makedirs(package_dir)

for from_path, to_path in PACKAGE_STATIC_CONTENT.items():
	# relativize pathes
	rel_from_path = os.path.join(os.getcwd(), from_path)
	rel_to_path = os.path.join(package_dir, to_path)
	# if folder tree doesnt exist make it
	if not os.path.exists(os.path.split(rel_to_path)[0]):
		os.makedirs(os.path.split(rel_to_path)[0])
	# copy file from->to
	try:
		shutil.copy2(rel_from_path, rel_to_path)
	except FileNotFoundError:
		pass

# create bin folder
binary_folder = os.path.join(package_dir, "bin")
os.makedirs(binary_folder)

# find the current default multirust toolchain so we can reset at the end
#   multirust show-default?
start_toolchain = s_Popen(["multirust", "show-default"], return_output=True).split("\n")[0].split(": ")[-1]

# build package in package_dir for arch + output package in OUTPUT_DIR
for t_a in TARGET_ARCHS:
	cargo_release_target = os.path.join(os.getcwd(), "target", "release", "theca")
	package_binary_path = os.path.join(binary_folder, "theca")
	# set multirust to the right toolchain
	current_toolchain = "-".join([RUST_CHANNEL, t_a, HOST_OS])
	s_Popen(["multirust", "default", current_toolchain])

	# build package
	build_output = s_Popen(BUILD_CMD, return_output=True)

	# copy binary to binary_folder
	shutil.copy2(cargo_release_target, package_binary_path)

	# piece together package_name and create tarball in OUTPUT_DIR
	package_name = "-".join([PACKAGE_PREFIX, t_a, HOST_OS])
	package_tarball_path = os.path.join(OUTPUT_DIR, package_name+".tar.gz")
	with tarfile.open(package_tarball_path, "w:gz") as tarball:
		tarball.add(package_dir, arcname=package_name)

	# delete arch binary
	os.remove(package_binary_path)

# teardown tmpdir
shutil.rmtree(TMPDIR)

# write build report for host triple + mb write out hashes?

# reset multirust toolchain
s_Popen(["multirust", "default", start_toolchain])
