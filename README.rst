===
TRO
===

|CircleCI| |CratesIO|

``tro`` is a Trello API client for the command line written in rust.

.. image:: assets/tro_show_board.png
   :width: 400

> NOTE that tro is still considered to be in development. Expect things to change suddenly and without warning
> until this notice is removed!

Installation
============

Currently, the only way to install is through cargo

::

   cargo install tro

Setup
=====

Run ``tro setup`` to setup tro for the first time.

Take a look at ``tro --help`` for a list of all available subcommands after that.

Available Subcommands
=====================

* setup: Setup tro for the first time
* show: Show an object (Board, List or Card)
* search: Search for Trello cards
* create: Create an object
* move: Move a card from one list to another
* open: Open an object that has been closed
* close: Close an object
* label: Apply or remove a label on a card
* url: Display the url of an object
* attach: Attach a file to a Card
* attachments: View attachments on a Card
* me: display currently logged in user

How it works
============

Most of the subcommands in this tool work by specifying one or more patterns in the form of:

::

    <board pattern> <list pattern> <card pattern>

A pattern is any valid regex pattern. You can specify simple patterns such as just specifying a substring too.

``tro`` then attempts to match the pattern you supplied with the available object(s):

* If ``tro`` does not manage to find a match for one or more if the items specified, then it will display an appropriate error.

* If ``tro`` manages to find a unique match for each of the items specified, then it will successfully display the object(s) you requested.

* If ``tro`` finds any of the patterns are matched with multiple possible items, then the tool will be unable to precisely determine which object you were referring to and do its best to explain why.

Usage Example
=============

Say we have a board named "TODO" with two lists named "today" and "done".

We can show the entire board by just specifying the board name:

.. image:: assets/tro_show_board.png
   :width: 400

Notice how by default patterns are case insensitive. You can make pattern matches case sensitive with the ``-c`` flag.

If we want to only see a specific list within the board, we can specify an additional list pattern:

.. image:: assets/tro_show_list.png
   :width: 400

If we want to show/edit a specific card, then we can also specify an additional card pattern.

Showing a card will open your default editor (specified by the ``EDITOR`` environment variable) so that you can edit the contents of the specified card.

For example, running ``tro show todo today rust`` would open as follows:

.. image:: assets/tro_show_card.png
   :width: 400

A card which has contents can be easily spotted by the ``[...]`` marker when viewing a board or list:

.. image:: assets/tro_card_contents.png
   :width: 400

Subcommands
===========

This section will explain some of the more useful subcommands in detail

Create Command
--------------

Create a Board, List or Card.

To create a new board, specify no patterns with ``create``.

::

    $ tro create
    Board name: TODO

To create a new list within a board, specify the board which the list will belong to as a pattern.

::

    $ tro create TODO
    List name: Today

To create a new card within a list, specify the board and the list
which the card will belong to as the two patterns:

::

    $ tro create TODO today
    Card name: Walk the dog

When creating a card, you can specify the ``--show`` flag to immediately edit the card once it has been created.

You can also specify one or more labels to assign to a card with
the ``--label`` flag.

::

     $ tro create TODO today --label fun
     Card name: Walk the dog
     Applied  Fun Times  label

Search Command
--------------

You can perform a search for cards using Trello's Search_ syntax

For example:

::

    $ tro search dog bones is:open
    Dig up some dog bones [...] id: 5ed78889acdaf970289ac894
    walk the dog id: 5da72eed111e6a56d3407e0b

All operators in the standard Trello search syntax are supported. For example if we want cards which
only have descriptions:

::

    $ tro search dog bones is:open has:description
    Dig up some dog bones [...] id: 5ed78889acdaf970289ac894

If you wish to use the negative operator, use ``~`` instead of ``-``.

::

    $ tro search dog bones is:open ~has:description
    walk the dog id: 5da72eed111e6a56d3407e0b

Interactive Mode
================

Most subcommands have an interactive mode that can be enabled by passing the ``--interactive`` or ``-i`` flag.

Interactive mode provides a simple keyboard interface to choose relative items when possible.

.. _Search: https://help.trello.com/article/808-searching-for-cards-all-boards

.. |CircleCI| image:: https://circleci.com/gh/MichaelAquilina/tro.svg?style=svg
   :target: https://circleci.com/gh/MichaelAquilina/tro

.. |CratesIO| image:: https://img.shields.io/crates/v/tro.svg
   :target: https://crates.io/crates/tro
