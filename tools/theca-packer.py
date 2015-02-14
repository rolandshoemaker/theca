#!/usr/bin/python
# -*- coding: utf-8 -*-
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# theca-packer.py
#   fabric based tool to package multiple different target arch/platform
#   combinations (only tested on x86_64/i686-linux/darwin combinations)
#   using multirust to manage rust toolchains (kinda hacky tbh...)

from __future__ import with_statement, print_function
from fabric.api import *
from fabric.utils import puts

from fabric.contrib.files import exists

import os, uuid, time, platform, json
from hashlib import sha256
from datetime import datetime

GIT_REPO = "https://github.com/rolandshoemaker/theca"
BUILD_CMD = "cargo build --release --verbose"
PACKAGE_STATIC_CONTENT = {
    "README.md": "README.md",
    "LICENSE": "LICENSE",
    "tools/package-installer.sh": "install.sh",
    "docs/THECA.1": "share/man/man1/theca.1",
    "completion/bash_complete.sh": "etc/bash_completion.d/theca",
    "completion/_theca": "share/zsh/site-functions/_theca"
}
TARGET_ARCHS = ["x86_64", "i686"]

MULTIRUST_INSTALL_CMD = "curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh"

SERVER_STATIC_DIR="/var/www/static/theca/dist"

def _run_mkdir(path):
    return run("mkdir -p %s" % (path))

def _where(command):
    with settings(warn_only=True):
        with hide("warnings", "output", "running"):
            if run("which %s" % (command)).return_code != 0:
                return False
            else:
                return True

# sometimes this works?
def _setup_toolchain(toolchain):
    # make sure toolchain is up to date
    toolchain_installer_url = "https://static.rust-lang.org/dist/rust-%s.tar.gz" % (toolchain)
    with hide("output"):
        puts("# %s: install/update" % (toolchain))
        run("multirust update %s --installer %s" % (toolchain, toolchain_installer_url))

@parallel
def check_ability():
    git = _where("git")
    multirust = _where("multirust")
    bad_native = not multirust and (_where("rustc") or _where("cargo"))
    if not bad_native:
        rustc = _where("rustc")
        cargo = _where("cargo")
    tar = _where("tar")

    puts("# host ability")
    puts("#   has git: %s" % (git))
    puts("#   has multirust: %s" % (multirust))

    if bad_native:
        puts("#     - does have (multirust incompatible) native rust toolchain")
    else:
        puts("#   has proper rustc: %s" % (rustc))
        puts("#   has proper cargo: %s" % (cargo))

    # should check this is the right tar...
    puts("#   has tar: %s" % (tar))

    if multirust:
        with hide("output", "running"):
            toolchains = run("multirust list-toolchains")
            puts("# available toolchains")
            for l in toolchains.split("\n"):
                puts("#   %s" % (l))

@parallel
def install_toolchains(rust_channel, target_arch=None):
    with settings(warn_only=True):
        puts("# check if multirust is installed")
        if not _where("multirust"):
            puts("# nop, install multirust")
            # should check if rust/cargo is and run traditional uninstaller before this idk...?
            run(MULTIRUST_INSTALL_CMD)
        puts("# got it!")
        with hide("output", "running"):
            host_string = run("uname -s")
            puts("# guessing host os")
            if host_string == "Linux":
                host_os = "unknown-linux-gnu"
            elif host_string == "Darwin":
                host_os = "apple-darwin"
        targets = target_arch or TARGET_ARCHS
        if type(targets) == str:
            targets = [targets]
        puts("# setup toolchains for targets %s" % (targets))
        for t_a in targets:
            current_toolchain = "-".join([rust_channel, t_a, host_os])
            _setup_toolchain(current_toolchain)
    puts("# %s IS DONE \(◕ ◡ ◕\)" % (env.host))

@parallel
def all_toolchains():
    with hide("output", "running"):
        toolchains = run("multirust list-toolchains")
        puts("# available toolchains")
        for l in toolchains.split("\n"):
            puts("#   %s" % (l))

@parallel
def _packager(package_prefix, commit_hash, output_dir, clone_depth=50, rust_channel="nightly", target_arch=None):
    puts("# STARTED")

    # linux / os x agnostic tmpdir
    puts("# creating temporary directory")
    with hide("output"):
        host_tmp_dir = run("mktemp -d 2>/dev/null || mktemp -d -t 'theca-packer-tmp'")
        if host_tmp_dir in ["", "/", ""]:
            # just making sure...
            exit(1) # ?

    # get host triple stuff
    puts("# geuessing host os")
    with hide("output"):
        host_string = run("uname -s")
    if host_string == "Linux":
        host_os = "unknown-linux-gnu"
    elif host_string == "Darwin":
        host_os = "apple-darwin"

    s_log = ""

    # clone git repo
    puts("# cloning repo")
    clone_dir = os.path.join(host_tmp_dir, package_prefix+"_build")
    s_log += _run_mkdir(clone_dir)
    with cd(clone_dir):
        with hide("output"):
            s_log += run("git clone --depth=%d %s" % (clone_depth, GIT_REPO))
    git_dir = os.path.join(clone_dir, GIT_REPO.split("/")[-1])
    with cd(git_dir):
        with hide("output"):
            s_log += run("git checkout -qf %s" % (commit_hash))

    # make package dir
    puts("# building static package")
    package_dir = os.path.join(host_tmp_dir, package_prefix)
    s_log += _run_mkdir(package_dir)

    # build dir structure + copy static content
    for from_path, to_path in PACKAGE_STATIC_CONTENT.items():
        # relativize pathes
        rel_from_path = os.path.join(git_dir, from_path)
        rel_to_path = os.path.join(package_dir, to_path)
        # if folder tree doesnt exist make it
        if not os.path.exists(os.path.split(rel_to_path)[0]):
            s_log += _run_mkdir(os.path.split(rel_to_path)[0])
        # copy file from->to
        with hide("output"):
            s_log += run("cp %s %s" % (rel_from_path, rel_to_path))

    # make binary folder
    puts("# make binary folder")
    binary_dir = os.path.join(package_dir, "bin")
    s_log += _run_mkdir(binary_dir)

    # build package in package_dir for arch + return to master
    targets = target_arch or TARGET_ARCHS
    if type(targets) == str:
        targets = [targets]
    packages = []
    puts("# starting packager for arches %s" % (targets))
    for t_a in targets:
        p_log = ""
        p_started = time.time()
        current_toolchain = "-".join([rust_channel, t_a, host_os])
        puts("# %s-%s: started" % (package_prefix, current_toolchain))

        # make sure toolchain is up to date?
        # _setup_toolchain(current_toolchain)

        # set the right toolchain
        puts("# %s-%s: setting toolchain to %s" % (package_prefix, current_toolchain, current_toolchain))
        with cd(git_dir):
            with hide("output"):
                p_log += run("multirust override %s" % (current_toolchain))

        # build package
        puts("# %s-%s: building binary with '%s'" % (package_prefix, current_toolchain, BUILD_CMD))
        with cd(git_dir):
            with hide("output"):
                p_log += run(BUILD_CMD)
                p_log += run("multirust remove-override")

        # copy binary to binary_dir
        puts("# %s-%s: copying binary to package directory" % (package_prefix, current_toolchain))
        cargo_release_target = os.path.join(git_dir, "target", "release", "theca")
        package_binary_path = os.path.join(binary_dir, "theca")
        with hide("output"):
            p_log += run("cp %s %s" % (cargo_release_target, package_binary_path))

        # piece together package_name and create tarball in OUTPUT_DIR
        package_dir_name = "-".join([package_prefix, t_a, host_os])
        package_name = package_dir_name+".tar.gz"
        package_tarball_path = os.path.join(host_tmp_dir, package_name)

        # make tarball in host_tmp_dir with root dir package_name...
        puts("# %s-%s: creating tarball '%s'" % (package_prefix, current_toolchain, package_name))
        with cd(host_tmp_dir):
            with hide("output"):
                p_log += run("tar -czf %s %s --transform 's,^%s,%s,'" % (package_name, package_prefix, package_prefix, package_dir_name))

        # get tarball from remote
        puts("# %s-%s: copying tarball '%s' to master" % (package_prefix, current_toolchain, package_name))
        get(package_tarball_path, os.path.join(output_dir, package_name))

        # generate sha hash
        puts("# generating sha256 sum: %s" % (package_name+".sha256"))
        with open(os.path.join(output_dir, package_name+".sha256"), "w") as out_file:
            with open(os.path.join(output_dir, package_name), "rb") as in_file:
                package_hash = sha256(in_file.read()).hexdigest()
                out_file.write("  ".join([package_hash, package_name])+"\n")

        # delete arch binary and package
        puts("# %s-%s: deleting %s binary and package" % (package_prefix, current_toolchain, current_toolchain))
        with hide("output"):
            p_log += run("rm %s" % (package_binary_path))
            p_log += run("rm %s" % (package_tarball_path))

        puts("# %s-%s: finished packager" % (package_prefix, current_toolchain))

	packages.append({
            "package_name": package_name,
            "package_sha256": package_hash,
            "toolchain_used": current_toolchain,
            "packer_log": p_log,
            "packing_took": time.time()-p_started
        })

    puts("# finished packaging")

    # teardown tmpdir
    puts("# deleting temporary directory")
    with hide("output"):
        t_log = run("rm -rf %s" % (host_tmp_dir))

    puts("# %s BUILDING IS DONE \(◕ ◡ ◕\)" % (env.host))

    # write build report?
    return {
        "packer_platform": platform.platform(),
	"packages": packages,
	"setup_and_teardown_log": "%s\n[BUILDING+PACKAGING]\n%s" % (s_log, t_log),
    }

@runs_once
def package(package_prefix, commit_hash, output_dir, clone_depth=50, rust_channel="nightly", target_arch=None):
	report_name = "%s_build_report.json" % (package_prefix)
	packager_reports = execute(_packager, package_prefix, commit_hash, output_dir, clone_depth=clone_depth, rust_channel=rust_channel, target_arch=target_arch)
	full_report = {
		"package_prefix": package_prefix,
		"git_commit": commit_hash,
		"rust_channel": rust_channel,
		"packed_at_utc": datetime.now().isoformat(),
		"packer_reports": packager_reports
	}

	with open(os.path.join(output_dir, report_name), "w") as f:
		json.dump(full_report, f, indent=2)

	return full_report

def upload_to_static(build_report, update_installer=False):
	to_upload = [r["package_name"] for p in build_report["packer_reports"] for r in p["packages"]]
	to_upload.append("%s_build_report.json" % (build_report['package_prefix']))

	if exists(os.path.join(SERVER_STATIC_DIR, "%s_build_report.json" % (build_report['package_prefix']))):
		# move the current stuff to -> package_prefix-DD-MM-YY/
		with open(os.path.join(SERVER_STATIC_DIR, "%s_build_report.json" % (build_report['package_prefix']))) as old:
			old_report = json.load(old)
			dated_dir = "%s-%s" % (old_report["package_prefix"], old_report["packed_at_utc"][:10])
		_run_mkdir(os.path.join(SERVER_STATIC_DIR, dated_dir))
		for existing_upload in to_upload:
			run("mv %s %s" % (os.path.join(SERVER_STATIC_DIR, existing_upload), os.path.join(SERVER_STATIC_DIR, dated_dir, existing_upload)))
	for upload in to_upload:
		put(os.path.join(output_dir, upload), os.path.join(SERVER_STATIC_DIR, upload))

def package_and_upload():
	pass
