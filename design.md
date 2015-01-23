# Design document

rust memo/note taking tool with a json note profile file format

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
		        "title": "(THECA) add profile merging",
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

## display commands

    # theca # prints all items for default/set profile
    # theca 1
    1   short title                     U       21-12-14
    # theca view 2
    2   long title with body            S       10-12-14
        This is the body, you probably found it because you searched
        for something in the body!

	long:

    # theca
    id  (+) title                       status      date
    --------------------------------------------------------
    1   short title                     Urgent      21-12-14
    2   (+) long title with body        Started     21-12-14
    3   i dont have a status                        21-12-14

	condensed:

    # theca -c
    1   short title                     U       10-12-14
    2   (+) long title with body        S       10-12-14
    3   i dont have a status                    21-12-14

## add item commands

    # theca add "this is the title guys whoa"
    # theca add "a item with started status" -ss
    # theca add "a item with urgent status" -su
    # theca add "item with a body from string" -b "this is the body of the note, this is a pretty limited way of adding a body, but if you must"
    # theca add "item with a body from $EDITOR" -b # drop to editor specified in $EDITOR to create body (similar to git commit)
    # cat file | theca add "item with a body from pipe" -b
    # theca add "item with a body from pipe" -b <file 

## edit item commands

    # theca edit 1 -ss # started
    # theca edit 1 -su # urgent!
    # theca edit 1 -sn # no status

    # theca edit 2 "this is the new title for this item"
    # theca edit 2 # drop to $EDITOR to edit body
    # theca edit 2 -b "this is the new body for this item"

## search commands

    # theca -ft keyword # filter title
    3    something keywords yehhhh      Urgent      20-12-14
    10   short title with keyword!      Urgent      21-12-14

    # theca -ft "i dont have a status "
    3    i dont have a status                       21-12-14

    # theca -fb something
    2   long title with body            S           10-12-14
        This is the body, you probably found it because you searched
        for something in the body!

    # theca -fb "something in the body!"
    2   long title with body            S           10-12-14
        This is the body, you probably found it because you searched
        for something in the body!

    # theca -ftr "\d+" # filter title using regex
    3    something 12312323 yehhhh      Urgent      20-12-14
    10   10 short title with ints       Urgent      21-12-14

    # theca -fbr "(\d+[\w\s]+\n?)+"
    2   long title with body            S           10-12-14
        10 this is basic
        20 kind of
        30 silly regex
