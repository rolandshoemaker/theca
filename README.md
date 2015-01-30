
	#  _   _                    
	# | |_| |__   ___  ___ __ _
 	# | __| '_ \ / _ \/ __/ _` |
	# | |_| | | |  __/ (_| (_| |
	#  \__|_| |_|\___|\___\__,_|
	#

![example usage of theca](screenshots/1.png)

a simple command line note taking tool written in [*Rust*](http://www.rust-lang.org/).

## features

* multiple profile support
* plaintext or 256-bit AES encrypted profiles
* *JSON* profile format for easy scripting/integration
* table and condensed printing modes
* add/edit/delete notes
* add/edit note body using command line arguments, `STDIN`, or using the editor set in `$EDITOR`
  or `$VISUAL`
* note transfer between profiles
* note searching (title or body) using keyword or regex pattern

## installation

### from source

all we need to build theca is a copy of the `rustc` compiler and the `cargo` packaging tool which can
be downloaded directly from the [Rust website](http://www.rust-lang.org/install.html) or by running

	$ curl -s https://static.rust-lang.org/rustup.sh | sudo sh

to get the nightly binaries, once those have finished building we can clone and build `theca`

	$ git clone https://github.com/rolandshoemaker/theca.git
	...

	$ cd theca
	$ cargo build [--release]
	...

	$ sudo ./build.sh install [--man, --bash-complete, --zsh-complete]

the `cargo` flag `--release` enables `rustc` optimizations. for `install` the flag `--man`
will additionally install the man page and `--bash-complete` and `--zsh-complete` will additionally install the
bash or zsh tab completion scripts. `cargo` will automatically download and compile `theca`s dependencies
for you.

## usage

### first run

![new default profile](screenshots/2.png)

running `theca new-profile` will create the `~/.theca` folder as well as the default
note profile in `~/.theca/default.json`.

### add a note

![adding a basic note](screenshots/3.png)

## development

currently i (roland) am the only developer of `theca` so literally any other pair of eyes
looking at the codebase would be super useful, especially considering how recently i started
using Rust, so feel free to submit pull requests!

### `theca_test_harness.py`

`theca_test_harness.py` is a *relatively* simple python3 test harness for the compiled `theca` binary.
it reads in JSON files which describe test cases and executes them, providing relatively simple
information like passed/failed/time taken.

the harness can preform three different output checks, against
 * the resulting profile file
 * the JSON output of view, list, and search commands
 * the text output of add, edit, delete commands, etc

a JSON test file looks something like this

	{
	  "title": "GOOD DEFAULT TESTS",
	  "desc": "testing correct input with the default profile.",
	  "tests": [
	  	...
	  ]
	 }

a profile result test looks something like this

	{
      "name": "add note",
      "cmds": [
        ["new-profile"],
        ["add", "this is the title"]
      ],
      "result_path": "default.json",
      "result": {
        "encrypted": false,
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

a JSON output test looks something like this

	{
      "name": "list",
      "cmds": [
        ["new-profile"],
        ["add", "a title this is"],
        ["add", "another title this is"],
        ["-j"]
      ],
      "result_type": "json",
      "results": [
        null,
        null,
        null,
        [
          {
            "id": 1,
            "title": "a title this is",
            "status": "",
            "body": ""
          },{
            "id": 2,
            "title": "another title this is",
            "status": "",
            "body": ""
          }
        ]
      ]
    }

and a text output test looks something like this

	{
      "name": "new-profile",
      "cmds": [
        ["new-profile"]
      ],
      "result_type": "text",
      "results": [
        "creating profile 'default'\n"
      ]
    }

### bugs

`theca` almost certainly contains bugs, i haven't had the time to write as many test cases as are really
necessary to fully cover the codebase. if you find one, please submit a issue explaining how to trigger
the bug, and if you're really awesome a test case that exposes it.
