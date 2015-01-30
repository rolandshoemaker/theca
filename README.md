
	#  _   _                    
	# | |_| |__   ___  ___ __ _
 	# | __|  _ \ / _ \/ __/ _` |
	# | |_| | | |  __/ (_| (_| |
	#  \__|_| |_|\___|\___\__,_|
	#

![example usage of theca](screenshots/main.png)

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
* note searching (title or body using keyword or regex pattern)

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

![new default profile](screenshots/first_run.png)

`theca new-profile` will create the `~/.theca` folder as well as the default
note profile in `~/.theca/default.json`.

### add a note

![adding a basic note](screenshots/add_simple_note.png)

`theca add` will add a note to the default profile with no body or status.

### edit a note

![editing a notes status](screenshots/edit_statuses.png)

### delete a note



### transfer a note



### view a note

![view a note](screenshots/view_note.png)

![view a note using the short print style](screenshots/view_note_condensed.png)

### list all notes

![list all notes](screenshots/list_notes.png)

### search notes

![search notes by title using regex](screenshots/search_note_regex.png)

### non-default profiles

![new encrypted profile](screenshots/new_second_profile.png)

#### encrypted

![new encrypted profile](screenshots/new_encrypted_profile.png)

## development

currently there is only one developer of `theca`, myself, so literally any other pair of eyes
looking at the codebase would be super useful, especially considering how recently i started
using Rust, so feel free to submit pull requests!

### profile JSON format

as described in `schema.json` this is what a note profile might look like

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

### cryptographic design

`theca` uses the AES CBC mode symmetric cipher with a 256-bit key derived using *pbkdf2*
(using the sha-256 prf) with 2056 rounds salted with the sha-256 hash of the password
used for the key derivation (probably not the best idea).

#### basic python implementation

using python a key can be derived as such

	from hashlib import sha256
	from passlib.utils.pbkdf2 import pbkdf2

	key = pbkdf2(
        bytes(passphrase.encode("utf-8")),
        sha256(bytes(passphrase.encode("utf-8"))).hexdigest().encode("utf-8"),
        2056,
        32,
        "hmac-sha256"
    )

and the ciphertext can be decrypted like so

	from Crypto.Cipher import AES

	# the IV makes up the first 16 bytes of the ciphertext
	iv = ciphertext[0:16]
    decryptor = AES.new(key, AES.MODE_CBC, iv)
    plaintext = decryptor.decrypt(ciphertext[16:])

    # remove any padding from the end of the final block
    plaintext = plaintext[:-plaintext[-1]].decode("utf-8")

### `theca_test_harness.py`

`theca_test_harness.py` is a *relatively* simple python3 test harness for the compiled `theca` binary.
it reads in JSON files which describe test cases and executes them, providing relatively simple
information like passed/failed/time taken.

the harness can preform three different output checks, against
 * the resulting profile file
 * the JSON output of view, list, and search commands
 * the text output of add, edit, delete commands, etc

the python script has a number of arguments that may help

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


#### test file format

a JSON test file looks something like this

	{
	  "title": "GOOD DEFAULT TESTS",
	  "desc": "testing correct input with the default profile.",
	  "tests": [
	  	...
	  ]
	}

##### test formats

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

### bugs

`theca` almost certainly contains bugs, i haven't had the time to write as many test cases as are really
necessary to fully cover the codebase. if you find one, please submit a issue explaining how to trigger
the bug, and if you're really awesome a test case that exposes it.
