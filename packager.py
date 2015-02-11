import platform, os, shutil, tempfile, tarfile, argparse
from subprocess import Popen, PIPE

# *Should* be run on clean git clone from the root of the repo...

def s_Popen(cmd, return_output=False, **kwargs):
	p = Popen(cmd, stdout=PIPE, stderr=PIPE, **kwargs)
	output = p.communicate()[0].decode("utf-8")
	if p.returncode > 0:
		# raise or something idk?
		pass
	if return_output:
		return output

parser = argparse.ArgumentParser(description="")
parser.add_argument("PREFIX", help="package prefix")
parser.add_argument("COMMIT_HASH", help="commit to build from")
parser.add_argument("OUTPUT_DIR", help="directory to place packaged tarballs")
parser.add_argument("--rust-channel", default="nightly", help="rust release channel to use")
parser.add_argument("--target_arch")
parser.add_argument("--clone-depth", default="50")
args = parser.parse_args()

GIT_REPO = "https://github.com/rolandshoemaker/theca"

OUTPUT_DIR = args.OUTPUT_DIR

RUST_CHANNEL = args.rust_channel
TARGET_ARCHS = args.target_arch or ["x86_64", "i686"]
if platform.system() == "Linux":
	HOST_OS = "unknown-linux-gnu"
elif platform.system() == "Darwin":
	HOST_OS = "apple-darwin"
else:
    print("nop")
    exit(1)

PACKAGE_PREFIX = args.PREFIX
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

# clone git repo
s_Popen(["git", "clone", "--depth="+args.clone_depth, GIT_REPO], cwd=TMPDIR) # --branch too?
# checkout right commit
git_folder = os.path.join(TMPDIR, GIT_REPO.split("/")[-1])
s_Popen(["git", "checkout", "-qf", args.COMMIT_HASH], cwd=git_folder)

# build static package structure 
package_dir = os.path.join(TMPDIR, PACKAGE_PREFIX)
os.makedirs(package_dir)

for from_path, to_path in PACKAGE_STATIC_CONTENT.items():
	# relativize pathes
	rel_from_path = os.path.join(git_folder, from_path)
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
	cargo_release_target = os.path.join(git_folder, "target", "release", "theca")
	package_binary_path = os.path.join(binary_folder, "theca")

	# make sure toolchain is up to date
	current_toolchain = "-".join([RUST_CHANNEL, t_a, HOST_OS])
	toolchain_installer_url = "https://static.rust-lang.org/dist/rust-%s.tar.gz" % (current_toolchain)
	s_Popen(["multirust", "update", current_toolchain, "--installer", toolchain_installer_url])

	# set the right toolchain
	s_Popen(["multirust", "default", current_toolchain])

	# build package
	build_output = s_Popen(BUILD_CMD, return_output=True, cwd=git_folder)

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
