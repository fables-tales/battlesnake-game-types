# Battlesnake Game Types

![crates.io](https://img.shields.io/crates/v/battlesnake-game-types.svg)
* [docs](https://docs.rs/battlesnake-game-types/latest/battlesnake_game_types/)

A crate to represent game types in the game of [battlesnake](https://play.battlesnake.com)


## Usage

The most common usage is decoding and encoding wire representation data:

```rust
use battlesnake_game_types::wire_representation::Game;
let g: Result<Game, _> = serde_json::from_slice(&body);
```


There are other useful tools that you can find better documented in the crate docs
