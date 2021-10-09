# Rad1

[![Build Status](https://gitlab.com/slfotg/rad1/badges/master/pipeline.svg)](https://gitlab.com/slfotg/rad1)

A Simple Chess Engine in Rust

## About
This is a very simple (command-line only + single-threaded)
chess engine written in Rust to explore chess engine algorithms
and to actually learn the language.
This is just be a fun side project with no real intent of making
a chess engine that is in any way competetive. I just wanted to see
if I could write a chess engine that I couldn't beat (I'm not that good ~1200-1300).
It uses [chess](https://github.com/jordanbray/chess) for move generation
and board and game representation.
The [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
has been a great resource for learning how to do this along with
[The Rust Book](https://doc.rust-lang.org/stable/book/).

Some of the algorithms for this engine are:
* [NegaMax](https://www.chessprogramming.org/Negamax)
  with [Alpha-Beta pruning](https://www.chessprogramming.org/Alpha-Beta)
* [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening)
* [Null Move Pruning](https://www.chessprogramming.org/Null_Move_Pruning)
* [Check Extensions](https://www.chessprogramming.org/Check_Extensions)
* [Quiescence Search](https://www.chessprogramming.org/Quiescence_Search)
* [Transposition Tables](https://www.chessprogramming.org/Transposition_Table)
* [Zobrist Hashing](https://www.chessprogramming.org/Zobrist_Hashing)

There are still a lot of things I'd like to improve upon:
* Adding [UCI Protocol](https://www.chessprogramming.org/UCI) for it to play within
  UCI GUIs and against other chess engines
* Some kind of multi-threaded search to increase speed and depth
* A way to [ponder](https://www.chessprogramming.org/Pondering) so it's
  not sitting idle while the other player is making a move
* Time controls
* Better (any) memory handling... This can use a lot of memory
* Improving the way transposition tables are handled
* Better evaluation function.
  The evaluation function I used is really naive but does suprisingly well.
  Only piece values and position are currently used.

## Quickstart
[Install rust and cargo](https://www.rust-lang.org/tools/install)

It's better to build a release version of the code instead of just using `cargo run`

    ❯ cargo build --release

Help:

    ❯ ./target/release/rad1-cli --help
    Rad1 Chess Engine 0.2.1
    Sam Foster <slfotg@gmail.com>
    A Simple Chess Engine in Rust

    USAGE:
        rad1-cli <SUBCOMMAND>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        analyze    Analyze a single position
        help       Prints this message or the help of the given subcommand(s)
        play       Play against the chess engine from terminal

To play against the engine in the terminal:

    ❯ ./target/release/rad1-cli play

To evaluate a specific position from a FEN representation

    ❯ ./target/release/rad1-cli analyze --fen "r3k2r/1p3pp1/p1p4p/3pP3/1PP5/P2P1P2/2qnKQ1P/8 b kq - 7 28"
    1 - d2c4 = 145
    2 - d2e4 = 224
    3 - d2e4 = 234
    4 - d2e4 = 237
    5 - d2e4 = 32767
    6 - d2e4 = 32767
    7 - d2e4 = 32767
    8 - d2e4 = 32767
    Best move: d2e4
    Size: 17779
