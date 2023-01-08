# async-core

Standardizing rust async, one type at a time.

## Why?

Tokio and async-std are great, but wouldn't it be cool to have *one* API for them all?
That's what I'm trying to provide with async-core. It's kind of similar to rand-core: No
implementations, only traits and some helper structs.

## This isn't enough.

I agree! So far, async-core only has some *very* basic types, which will never ever be
enough; however, I'm not entirely sure what to make next and how to make it. I've got the
following on my TODO list:

- IO helpers and traits: These are sadly quite hard because there are so many different
  things that use IO which need implementations, but I also don't want to mandate all
  libraries having IO operations for everything, also it is hard to keep that relatively
  small. I won't be able to let implementing be done fully by the runtime because then,
  nothing will compile (The goal of this is letting library users choose the runtime they
  want, regardless of what the library planned for, but for that, the library has to not
  depend on any runtime, meaning optional implementations won't be there, causing the
  compilation to fail because a trait isn't implemented. This wouldn't be a problem if
  crates.io didn't mandate a working compilation for a crate to be uploaded.).
- More utility functions runtimes need to implement
- Slightly better documentation

If anything is missing that you need *now*, make an issue! If you are the maintainer of a
runtime, please reach out and we can work out an implementation for your runtime and add
more to the standard.

## Warning: This is subject to rapid change.

This library is not stable, and there will be drastic changes, potential renames of
functions, etc. That also means if you see anything you have an issue with, tell me, and
there is still a high chance of me accepting your suggestion.

Releases every week if there are enough improvements until things stabelize some more,
then releases once changes have accumulated.
