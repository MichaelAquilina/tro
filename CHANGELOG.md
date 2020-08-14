Changelog for tro
=================

2.8.0
-----
* Add the move subcommand for moving to a different list in the same board

2.7.0
-----
* Fail create subcommand earlier if labels specified are not found

2.6.1
-----
* Fix minor issue in Cargo.toml which prevented upload to crates.io

2.6.0
-----
* Remove extra call to trello API in show subcommand for better performance
* Upgrade underlying reqwests library to 0.10.7

2.5.1
-----
* Fix some cases where reqwest Client was not being re-used
* Exit with code 2 when exiting with ctrl-c

2.5.0
-----
* Improve performance for all subcommands by re-using reqwest Client
* Fix bug where empty search results would crash tro
* Allow use of negative operators in search using "~"

2.4.0
-----
* Add me subcommand to show currently logged in user
* Display helpful error message tro configuration can not be loaded
* Add more detail to search subcommand help

2.3.1
-----
* Fix crash in setup subcommand when config dir does not exist

2.3.0
-----
* Validate provided credentials during setup
* Improved README

2.2.0
-----
* Add setup subcommand
* Provide default for host field in configuration file
* Update dependencies

2.1.0
-----
* Correctly calculate header widths with unicode text
* Update dependencies
* Fix bug where cursor would not show after pressing ctrl-c in interactive mode
* Improve search subcommand output

2.0.0
-----
* search subcommand now only searches for cards
* Add interactive mode to search subcommand
* Fix minor bug with label rendering when applying and deleting labels

1.34.0
------
* Use colored backgrounds with white text for labels (Similar to trello Web)

1.33.0
------
* Use the exact same label colors as trello web

1.32.0
------
* Upgrade colored dependency to 2.0.0

1.31.0
------
* Retrieve card due date

1.30.0
------
* Add --cards-limit and --boards-limit flags to search subcommand

1.29.0
------
* Render Cards and Labels with full colors in interactive mode

1.28.1
------
* Fix bug in select prompts where escape key would not register
* Remove dependency on simple-error

1.28.0
------
* Allow multi select in show subcommand
* Add interactive (multi-select) mode to labels subcommand
* Allow closing of boards interactively

1.27.0
------
* Allow multiple labels to be specified with the label subcommand

1.26.0
------
* Remove --show from close subcommand due to buggy behaviour
* Add --interactive mode to close and show subcommands

1.25.0
------
* Add ability to assign labels during create subcommand
* Display if a board is closed in search subcommand results

1.24.1
------
* Small improvements to error messages when editing cards
* Correctly retry card edits if parse error is detected and editor is closed

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
* Lots of internal re-organisation of code

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
