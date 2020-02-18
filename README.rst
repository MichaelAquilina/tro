Trello-RS
=========

|CircleCI| |CratesIO|

Trello API client for the command line written in rust.

To begin, create a configuration file at the path ``~/.config/tro/config.toml``.
Set the values for ``host``, ``key`` and ``token``:

::

    host = "https://api.trello.com"
    key = "<MYKEY>"
    token = "<MYTOKEN>"

You can retrieve the values for key and token from https://trello.com/app-key/

Once those are set, simply run with ``cargo run``.

How it works
------------

Most of the subcommands in this tool work by specifying one or more patterns in the form of:

::

    <board> <list> <card>

Patterns are simple regex pattern matches. You can specify simple patterns such as substrings too.

Trello-rs then attempts to find the object(s) you requested using this process:

* If the tool does not manage to find a match for one or more if the items specified, then it will display an appropriate error.

* If the tool manages to find a unique match for each of the items specified, then it will successfully display the object(s) you requested.

* If one or more of the patterns are matched with multiple possible items, then the tool will fail to retrieve the object(s) you requested and do its best to explain why.

Commands
--------

Some examples of commands you can run:

**Show all board names**

::

    $ tro show
    * TODO
    * Groceries
    * Recipes


**Show a specific board**

::

    $ tro show todo
    TODO
    ====

    Today
    -----
    * Wash Dishes
    * Walk Dog
    * Learn some Rust


**Create a new card**

::

    $ tro create TODO Today
    Card name: my new card

OR

::

    $ tro show TODO Today -n
    <your $EDITOR will open>

**Edit an existing card**

::

    $ tro show TODO Today "my new card"
    <your $EDITOR will open>

**Close a card**

::

    $ tro close TODO Today "my new card"
    Closed card 'my new card'

**Show a url**

::

    $ tro url TODO
    https://trello.com/b/9ftbid5U/todo

.. |CircleCI| image:: https://circleci.com/gh/MichaelAquilina/trello-rs.svg?style=svg
   :target: https://circleci.com/gh/MichaelAquilina/trello-rs

.. |CratesIO| image:: https://img.shields.io/crates/v/trello-rs.svg
   :target: https://crates.io/crates/trello-rs
