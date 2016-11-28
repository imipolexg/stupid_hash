# Stupid Hash

A hash implementation in Rust, based on a hash implementation in C based on the
hash implementation in Kernighan and Pike's *The Practice of Programming*
(1999).

I wanted to use arrays instead of vectors but ran into trouble with the Copy
trait.
