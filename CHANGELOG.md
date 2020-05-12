Changelog for trello-rs
=======================

1.24.0
------
* Rename crate to "tro"
* Update dependencies (including a minor bump to rustyline 6.1)
* Add --name flag to create subcommand to skip input prompt

1.23.0
------
* Display if a card is closed in search subcommand results

1.22.0
------
* Don't crash when writing cards if content is not parseable
* Update dependencies (patch version only)
* Update mockito to 0.25 for testing

1.21.1
------
* Display error details if --log-level=DEBUG

1.21.0
------
* Add first implementation of the search subcommand
* Allow readline style movements when entering text for prompts
* Add missing WARN level to --log-level flag
* Display possible log levels in CLI help
* Lot's of internal re-organisation of code

1.20.0
------
* Add --show flag to close subcommand
* Small improvements to error handling

1.19.0
------
* Saving a card being edited while the editor is still open will now upload contents in the background
* Remove --new flag for show subcommand
* Add --show flag to create subcommand
* Fix bug where --case-sensitive flag would not work

1.18.0
------
* Add attachments subcommand
* Add attach subcommand

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
