Changelog for trello-rs
=======================

1.17.0
------
* Use POST and PUT body to send data when applicable

1.16.0
------
* Use post and put body for creating/updating cards

1.15.1
------
* Fix crash in edit card retry mechanism

1.15.0
------
* Add --delete flag for label subcommand

1.14.0
------
* Add label subcommand

1.13.1
------
* Fix regression preventing correct editing/creating of cards using -n

1.13.0
------
* Add url subcommand
* Specify type in output when an issue with matching patterns occurs

1.12.0
------
* Color output when closing items
* Small improvements to error handling
* Display the ids of items that are closed
* Allow retries of updating/creating cards

1.11.0
------
* Better board titles
* Pattern matching is now case insensitive by default
* Display matched patterns when multiple found

1.10.1
------
* Fix regression detecting when a card is changed during an edit

1.10.0
-----
* Changelog introduced
* Add wildcard pattern for list names
