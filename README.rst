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

**Show all board**

::

    $ cargo run show
    * TODO
    * Groceries
    * Recipes


**Show a specific board**

::

    $ cargo run show TODO
    TODO
    ====

    Today
    -----
    * Wash Dishes
    * Walk Dog
    * Learn some Rust


**Create a new card**

::

    $ cargo run create TODO Today
    Card name: my new card

**Edit an existing card**

::

    $ cargo run show TODO Today "my new card"
    <your $EDITOR will open>

**Close a card**

::

    $ cargo run close TODO Today "my new card"
    Closed card 'my new card'

.. |CircleCI| image:: https://circleci.com/gh/MichaelAquilina/trello-rs.svg?style=svg
   :target: https://circleci.com/gh/MichaelAquilina/trello-rs

.. |CratesIO| image:: https://img.shields.io/crates/v/trello-rs.svg
   :target: https://crates.io/crates/trello-rs
