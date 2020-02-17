[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=Z8QK6XU749JB2) 
![Lines of Code][loc-badge]
[![MIT][license-badge]][license-link] 

# Example game legion entity synchronization.
This library contains an example game, which uses legion-sync crate to synchronize entities. 

The game is an simple terminal application in which a client can move around. 
The position of the client is sent to the server which in turn renders the client.

To run this game, clone the repository, and run `./start.sh` which will fire up the server and a single client.

[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-link]: ./docs/LICENSE

[loc-badge]: https://tokei.rs/b1/github/entity-sync-rs/legion-sync?category=code