# Design document

initial design document for *theca*, may not be completely up to date with the
current version.

## json note profile file

    as described in `schema.json` this is what the note profile looks like

    {
        "encrypted": false,
        "notes": [
            {
                "id": 2,
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
            },
            {
                "id": 5,
                "title": "(THECA) check about drop_to_editor adding newlines?",
                "status": "",
                "body": "",
                "last_touched": "2015-01-22 15:31:14 -0800"
            }
        ]
    }

## display note commands

    long note list:
    # theca
    id  title                          status      date
    -------------------------------------------------------
    1   short title                    Urgent      21-12-14
    2   long title with body      (+)  Started     21-12-14
    3   i dont have a status                       21-12-14

    condensed note list:

    # theca -c
    1   short title                     U       10-12-14
    2   long title with body       (+)  S       10-12-14
    3   i dont have a status                    21-12-14

    view single note:

    # theca 1
    2   long title with body            S       10-12-14
        This is the body!

## add note commands

    # theca add "this is the title guys whoa"
    # theca add "a item with started status" -s
    # theca add "a item with urgent status" -s
    # theca add "item with a body from string" -b "this is the body of the note, this is a pretty limited way of adding a body, but if you must"
    # theca add "item with a body from $EDITOR" --editor # drop to editor specified in $EDITOR to create body (similar to git commit)
    # cat file | theca add "item with a body from pipe" -
    # theca add "item with a body from pipe" - <file 

## edit note commands

    # theca edit 1 -s     # started
    # theca edit 1 -u     # urgent
    # theca edit 1 -n     # no status

    # theca edit 2 "this is the new title for this item"
    # theca edit 2 "a different title" --editor # drop to $EDITOR to edit body
    # theca edit 2 -b "just the body"
    # theca edit 2 "body from stdin" - <file

## delete note commands

    # theca del 2         # delete a single note

    # theca clear         # delete all notes

## search note commands

    # theca search keyword                        # search titles for keyword
    # theca search --regex /\d+\/                      # search titles using regex pattern
    # theca search --search-body keyword          # search bodies using keyword
    # theca search --regex --search-body /\d+\/        # search bodies using regex pattern

## encryption options
	
	# theca -p encrypted_profile -k THE_KEY
	# theca -p encrypted_profile -e               # get key from STDIN
