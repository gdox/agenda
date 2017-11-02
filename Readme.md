= Todo list

Simple implementation of a todo list/agenda.

= Installation

Currently the app uses `/etc/agenda/agenda.conf` as configuration, which should be a file like the agenda.conf in the root. It contains only one entry, namely `database`, which is the file containing all the stuff.

The program doesn't yet create the file because honestly I have no idea how to do "create file if it doesn't exist" stuff yet. Hints are always welcome.

Also try to put the executable in /usr/local/bin/agenda for fast access.

= Usage

Easy:
```
cargo run -- add Make a todo list -d "10 10 2100"
```
or, after copying the executable:
```
agenda add Make a todo list -d "1 10 2100"
```
Makes a new item in the todo list with deadline 1 October 2100. The format is `<D> <M> <Y> [h] [m]] [s]`

```
agenda list
```
shows what you have to do, with the `-t` option putting the item with the closest deadline on top and `-s`going for a one-line-per-item output, and
```
agenda delete 5
```
deletes the item that starts with "Event 5".

= License.

I hacked this thing together in three hours, quickly wrote a readme, and posted it online. Do you really think I had the time to look into licenses? I needed the functionality, kay?

If my code looks ugly and you know a better way, send me a notif.

If my code wipes your hard drive (No idea how that could ever happen since everyone has an SSD nowadays), I'm not to blame.

If you think I'm awesome and you'll devote your entire life and income to my happiness: I'm flattered. Thank you.

If you hate me and want me to die: I'm not flattered at all.

= Comments

Seriously, any comments about how I could improve the code, the readme, or my love life are more than welcome.

PS Does anyone know a crate that converts human readable dates (like 'next wednesday at 5pm') into a fixed date? Python has such thing; Rust apparently doesn't.
