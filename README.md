
	#  _   _                    
	# | |_| |__   ___  ___ __ _
 	# | __|  _ \ / _ \/ __/ _` |
	# | |_| | | |  __/ (_| (_| |
	#  \__|_| |_|\___|\___\__,_|
	#

![example usage of theca](screenshots/main.png)

a simple command line note taking tool written in [*Rust*](http://www.rust-lang.org/).

## Features

* Multiple profile support
* Plaintext or 256-bit AES encrypted profiles
* *JSON* profile format for easy scripting/integration
* Traditional and condensed printing modes
* Add/edit/delete notes
* Add/edit note body using command line arguments, `STDIN`, or using the editor set in `$EDITOR`
  or `$VISUAL`
* Note transfer between profiles
* Note searching (title or body using keyword or regex pattern)

## Contents

- [Installation](#)
	- [From source](#)
- [Usage](#)
	- [First run](#)
	- [Add a note](#)
	- [Edit a note](#)
	- [Delete a note](#)
	- [Transfer a note](#)
	- [View a note](#)
	- [List all notes](#)
	- [Search notes](#)
	- [Non-default profiles](#)
		- [Encrypted profiles](#)
- [Tab completion](#)
- [Development](#)
	- [JSON profile format](#)
	- [Cryptographic design](#)
		- [Basic Python implementation](#)
	- [theca_test_harness.py](#)
		- [Test suite file format](#)
			- [Test formats](#)
	- [Bugs](#)

## Installation

### From source

All that's needed to build theca is a copy of the `rustc` compiler and the `cargo` packaging tool which can
be downloaded directly from the [Rust website](http://www.rust-lang.org/install.html) or by running

	$ curl -s https://static.rust-lang.org/rustup.sh | sudo sh

to get the nightly binaries, once those have finished building we can clone and build `theca`

	$ git clone https://github.com/rolandshoemaker/theca.git
	...

	$ cd theca
	$ cargo build [--release]
	...

	$ sudo ./build.sh install [--man, --bash-complete, --zsh-complete]

The `cargo` flag `--release` enables `rustc` optimizations. for `install` the flag `--man`
will additionally install the man page and `--bash-complete` and `--zsh-complete` will additionally install the
bash or zsh tab completion scripts. `cargo` will automatically download and compile `theca`s dependencies
for you.

## Usage

	$ theca --help
	theca - simple cli note taking tool

	Usage:
	    theca [options] new-profile [<name>]
	    theca [options] info
	    theca [options] clear
	    theca [options]
	    theca [options] <id>
	    theca [options] search [--regex, --search-body] <pattern>
	    theca [options] transfer <id> to <name>
	    theca [options] transfer-from <id> from <name>
	    theca [options] add <title> [-s|-u] [-b BODY|-t|-]
	    theca [options] edit <id> [<title>] [-s|-u|-n] [-b BODY|-t|-]
	    theca [options] del <id>...

	Profiles:
	    -f PATH, --profile-folder PATH      Path to folder containing profile.json
	                                        files [default can be set with env var 
	                                        THECA_PROFILE_FOLDER].
	    -p PROFILE, --profile PROFILE       Specify non-default profile [default
	                                        can be set with env var 
	                                        THECA_DEFAULT_PROFILE].

	Printing format:
	    -c, --condensed                     Use the condensed printing format.
	    -j, --json                          Print list output as a JSON object.

	Note list formatting:
	    -l LIMIT, --limit LIMIT             Limit output to LIMIT notes.
	                                        [default: 0].
	    -d, --datesort                      Sort notes by date.
	    -r, --reverse                       Reverse list.

	Input:
	    -y, --yes                           Silently agree to any [y/n] prompts.

	Statuses:
	    -n, --none                          No status. (note default)
	    -s, --started                       Started status.
	    -u, --urgent                        Urgent status.

	Body:
	    -b BODY, --body BODY                Set body of the note to BODY.
	    -t, --editor                        Drop to $EDITOR to set/edit note body.
	    -                                   Set body of the note from STDIN.

	Encryption:
	    -e, --encrypted                     Specifies using an encrypted profile.
	    -k KEY, --key KEY                   Encryption key to use for encryption/
	                                        decryption, a prompt will be
	                                        displayed if no key is provided.

	Search:
	    --search-body                       Search the body of notes instead of
	                                        the title.
	    --regex                             Set search pattern to regex (default
	                                        is plaintext).

	Miscellaneous:
	    -h, --help                          Display this help and exit.
	    -v, --version                       Display the version of theca and exit.

### First run

![new default profile](screenshots/first_run.png)

`theca new-profile` will create the `~/.theca` folder as well as the default
note profile in `~/.theca/default.json`.

### Add a note

![adding a basic note](screenshots/add_simple_note.png)

`theca add` will add a note to the default profile with no body or status.

### Edit a note

![editing a notes status](screenshots/edit_statuses.png)

### Delete a note



### Transfer a note



### View a note

![view a note](screenshots/view_note.png)

![view a note using the short print style](screenshots/view_note_condensed.png)

### List all notes

![list all notes](screenshots/list_notes.png)

### Search notes

![search notes by title using regex](screenshots/search_note_regex.png)

### Non-default profiles

![new encrypted profile](screenshots/new_second_profile.png)

#### Encrypted profiles

![new encrypted profile](screenshots/new_encrypted_profile.png)

## Tab completion

There are preliminary `bash` and `zsh` tab completion scripts in the `completion/` directory
that can be installed manually or using the `--bash-complete` or `--zsh-complete` flags with
`sudo ./build.sh install`. They both need quite a bit of work but are still relatively usable.

## Development

Currently there is only one developer of `theca`, myself, so literally any other pair of eyes
looking at the codebase would be super useful, especially considering how recently I started
using Rust, so feel free to submit pull requests!

### JSON profile format

As described much more verbosely in `schema.json`, this is what a note profile might look like

    {
        "encrypted": false,
        "notes": [
            {
                "id": 1,
                "title": "\\(◕ ◡ ◕\\)",
                "status": "",
                "body": "",
                "last_touched": "2015-01-22 15:01:39 -0800"
            },
            {
                "id": 3,
                "title": "(THECA) add super secret stuff",
                "status": "",
                "body": "",
                "last_touched": "2015-01-22 15:21:01 -0800"
            }
        ]
    }

### Cryptographic design

`theca` uses the AES CBC mode symmetric cipher with a 256-bit key derived using *pbkdf2*
(using the sha-256 prf) with 2056 rounds salted with the sha-256 hash of the password
used for the key derivation (probably not the best idea).

#### Basic Python implementation

Using `python3` a key can be derived quite quickly using `hashlib` and `passlib`

	from hashlib import sha256
	from passlib.utils.pbkdf2 import pbkdf2

	key = pbkdf2(
        bytes(passphrase.encode("utf-8")),
        sha256(bytes(passphrase.encode("utf-8"))).hexdigest().encode("utf-8"),
        2056,
        32,
        "hmac-sha256"
    )

and the ciphertext can be decrypted using the AES implementation from `pycrypto`

	from Crypto.Cipher import AES

	# the IV makes up the first 16 bytes of the ciphertext
	iv = ciphertext[0:16]
    decryptor = AES.new(key, AES.MODE_CBC, iv)
    plaintext = decryptor.decrypt(ciphertext[16:])

    # remove any padding from the end of the final block
    plaintext = plaintext[:-plaintext[-1]].decode("utf-8")

### `theca_test_harness.py`

`theca_test_harness.py` is a *relatively* simple python3 test harness for the compiled `theca` binary.
It reads in JSON files which describe test cases and executes them, providing relatively simple
information like passed/failed/time taken.

The harness can preform three different output checks, against
 * the resulting profile file
 * the JSON output of view, list, and search commands
 * the text output of add, edit, delete commands, etc

The python script has a number of arguments that may or may not be helpful

	$ python3 tests/theca_test_harness.py -h
	usage: theca_test_harness.py [-h] [-tc THECA_COMMAND] [-tf TEST_FILE] [-pt]
	                             [-jt] [-tt]

	test harness for the theca cli binary.

	optional arguments:
	  -h, --help            show this help message and exit
	  -tc THECA_COMMAND, --theca-command THECA_COMMAND
	                        where is the theca binary
	  -tf TEST_FILE, --test-file TEST_FILE
	                        path to specific test file to run
	  -pt, --profile-tests  only run the profile output tests
	  -jt, --json-tests     only run the json output tests
	  -tt, --text-tests     only run the text output tests


#### Test suite file format

A JSON test suite file looks something like this

	{
	  "title": "GOOD DEFAULT TESTS",
	  "desc": "testing correct input with the default profile.",
	  "tests": [
	  	...
	  ]
	}

##### Test formats

* a profile result test looks something like this

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

* a JSON output test looks something like this

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

* a text output test looks something like this

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

### Bugs

`theca` almost certainly contains bugs, I haven't had the time to write as many test cases as are really
necessary to fully cover the codebase. if you find one, please submit a issue explaining how to trigger
the bug, and if you're really awesome a test case that exposes it.
