# `theca` cli test harness

this folder contains the python harness for testing the `theca` cli binary as well
as the `.json` files containing the tests to administer. it currently only checks a
single resulting profile.

it should, in the future, have multiple result paths, result objects as well as 
being able to check the json output (from -j/--json) for list/search commands!

also the profile folder stuff should be fixed, add a `travis-tests.json` file that
will contain all the home-dir based tests, all the other profiles should specify
profile-folder manually.

should also add a `should_fail` field for tests that should fail with errors?
/ check error code or something idk.

also tests that dont both with a passphrase should prob just not have the field, we
don't need to adhere to having empty fields in every test.

## example test file

	{
	  "title": "SOME TESTS",
	  "desc": "what do these tests test again?",
	  "tests": [
	    {
	      "name": "test_command",
	      "profile": "",
	      "profile_folder": "",
	      "cmds": [
	        ["new-profile"]
	      ],
	      "stdin": [],
	      "result_path": "default.json",
	      "result_passphrase": "",
	      "result": {
	        "encrypted": false,
	        "notes": []
	      }
	    }
	  ]
	}

