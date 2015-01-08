# theca

## road to 0.9 (first release)

**1.0** will be come out with *rust* 1.0-stable so i dont have to change stuff every day

* ~~note body from STDIN (easy)~~
* ~~strip newlines from title (easy)~~
* ~~note search functions (this'll be annoying :/)~~
 * ~~Regex~~
* ~~better LineFormat using termsize to set max widths etc... (annoying)~~ *(p. much)*
* ~~finish printing (e/c) (easy...ish)~~ *(p. much)*
* **implement aesc/desc sort**
* **implement sep by day/week/month**
* encryptable profiles (unsure of which crypto library to use, rust-crypto most likely)
* proper setup functions for first time use (p easy)
* better status handling (hooooow)
* (subjective) lots of clean-up (:>)

## theca design

rust memo/note taking tool with a json note file format

## json note file

    {
        "encrypted": false,
        "notes": [
            {
                "id": 1,
                "title": "short title",
                "body": "",
                "status": "Urgent",
                "last_touched": "2015-01-08 21:42:49"
            },{
                "id": 2,
                "title": "long title with body",
                "body": "wow we can hold some stuff in here that we might want to read later\nwhoa that was a linebreak huh!\n\nyes, yes it was",
                "status": "Started",
                "last_touched": "2014-12-28 18:36:23"
            },{
                "id": 3,
                "title": "i dont have a status or a body",
                "body": "",
                "status": "",
                "last_touched": "2015-01-08 21:43:26"
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

## features

* allow import from one profile file to another?
* allow encrypted note file (encrypt all fields or just title/body/status/last_touched? i.e. leave current_id and id free...)
* multiple profiles (default+selectable profiles)
* drop to editor to add/edit long notes / commandline for just titles
* stdin to new note
* read file to new note
* store user config in `~/.thecarc` (header/condensed, .theca/ (dropbox/~), etc)
* store note files in `~/.thecha/`
* not rly a feature but storing `.theca` folder in dropbox should sync pretty well...

### display

* print from provided profile
* print from default profile
* print from all profiles
* print with header/condensed
* show REAL id / show RELATIVE id
* print in order
* print by day/week/month last touched

### searching

* search by title keyword
* search by body keyword
* search both by user provided regex

### per item info

* id (incr int)
* status (urgent/started/) finished seems unnessacary...
* title
* optional text body
* datetime last touched (created/edited)
