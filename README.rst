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

Some examples of commands you can run:

**List all open boards**
::

    $ cargo run boards ls
    TODO
    Groceries
    Recipes


**List all open lists within a board**
::

    $ cargo run boards get -n "TODO" lists ls
    Today
    Tomorrow


**List all open cards within a list**
::

    $ cargo run boards get -n "TODO" lists get -n "Today" cards ls
    Wash Dishes
    Walk Dog
    Learn some Rust

.. |CircleCI| image:: https://circleci.com/gh/MichaelAquilina/trello-rs.svg?style=svg
   :target: https://circleci.com/gh/MichaelAquilina/trello-rs

.. |CratesIO| image:: https://img.shields.io/crates/v/trello-rs.svg
   :target: https://crates.io/crates/trello-rs
