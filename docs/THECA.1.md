THECA 1 "2015" theca v1.0-alpha THECA
=====================================

NAME
----

theca - minimal cli note taking tool

SYNOPSIS
--------

`theca` [`options`]

`theca` [`options`] <`id`>

`theca` [`options`] add <`title`> [`-s`|`-u`] [`-b` *BODY*|`-t`|`-`]

`theca` [`options`] edit <`id`> [<`title`>] [`-s`|`-u`|`-n`] [`-b` *BODY*|`-t`|`-`]

`theca` [`options`] del <`id`>

`theca` [`options`] new-profile [<`name`>]

`theca` [`options`] info

`theca` [`options`] clear

`theca` [`options`] search [`--regex`, `--search-body`] <`pattern`>

`theca` [`options`] transfer <`id`> to <`name`>

DESCRIPTION
-----------

`theca` is a minimal command line profile based note taking tool
written in `rust` that stores profiles using a `JSON` based file
format.

PROFILE OPTIONS
---------------

`-f` *PATH*, `--profile-folder` *PATH*
   Path to folder containing profile.json files, this override
   any `THECA_DEFAULT_PROFILE` environment variable.

`-p` *PROFILE*, `--profile` *PROFILE*
   Specify non-default profile [default can be set with env var 
   `THECA_DEFAULT_PROFILE`].

PRINTING OPTIONS
----------------

`-c`, `--condensed`
   Use the condensed printing format.

LIST OPTIONS
------------

`-l` *LIMIT*, `--limit` *LIMIT*
   Limit listing to LIMIT items [default: 0].

`-r`, `--reverse`
   Reverse list.

`-d`, `--datesort`
   Sort items by date, can be used with --reverse.

INPUT OPTIONS
-------------

`-y`, `--yes`
   Silently agree to any [y/n] prompts.

STATUS OPTIONS
--------------

`-n`, `--none`
   No status. (note default)

`-s`, `--started`
   Started status.

`-u`, `--urgent`
   Urgent status.

BODY OPTIONS
------------

`-b` *BODY*, `--body` *BODY*
   Set body of the item to BODY.

`-t`, `--editor`
   Drop to `EDITOR` to set/edit item body.

`-`
   Set body of the item from STDIN.

ENCRYPTION OPTIONS
------------------

`-e`, `--encrypted`
   Specifies using an encrypted profile.

`-k` *KEY*, `--key` *KEY*
   Encryption key to use for encryption/decryption, a prompt
   will be displayed if no key is provided.

SEARCH OPTIONS
--------------

`--search-body`
   Search the body of notes instead of the title.

`--regex`
   Set search pattern to regex (default is plaintext).

MISC OPTIONS
------------

`-h`, `--help`
   Display this help and exit.

`-v`, `--version`
   Display the version of theca and exit.

FILES
-----

*~/.theca/default.json~
   The default profile file that `theca` attempts to read.

ENVIRONMENT
-----------

`THECA_DEFAULT_PROFILE`
   If non-null the default profile for `theca` to read. Overridden by
   the `-p` option.

`THECA_PROFILE_FOLDER`
   If non-null the full path for for the theca profile `folder`.
   Overridden by the `-f` option.

FILE FORMAT
-----------

`theca` uses a `JSON` based file format that adheres to the following
schema.

   {
    "$schema": "https://github.com/rolandshoemaker/theca/blob/master/docs/DESIGN.md",
    "id": "/",
    "type": "object",
    "properties": {
      "encrypted": {
        "id": "encrypted",
        "type": "boolean"
      },
      "notes": {
        "id": "notes",
        "type": "array",
        "items": {
          "id": "0",
          "type": "object",
          "properties": {
            "id": {
              "id": "id",
              "type": "integer"
            },
            "title": {
              "id": "title",
              "type": "string"
            },
            "status": {
              "id": "status",
              "type": "string"
            },
            "body": {
              "id": "body",
              "type": "string"
            },
            "last\_touched": {
              "id": "last\_touched",
              "type": "string"
            }
          },
          "additionalProperties": false,
          "required": [
            "id",
            "title",
            "body",
            "last_touched"
          ]
        },
        "additionalItems": false
      }
    },
    "additionalProperties": false,
    "required": [
      "encrypted",
      "notes"
    ]
   }

AUTHOR
------

Roland Bracewell Shoemaker <rolandshoemaker@gmail.com>

SEE ALSO
--------

memo(1)
