# Invader (old version)

This is the initial Rust rewrite of Invader, started in 2022.

Main development has moved to [ringhopper](https://github.com/FishAndRips/ringhopper),
however this version may still see some small fixes and additions while the remaining
features are implemented.

The original C++ project can be found on https://github.com/SnowyMouse/invader

Invader is licensed under version 3 of the GNU General Public License. See
LICENSE.txt for more information.


## Structure

All source code is located in the `src` folder.

- `src/invader` - Command line frontend
- `src/ringhopper` - Rust library for modding Halo: Combat Evolved.


## Why is Invader being rewritten?

Invader has become very difficult to maintain for me, and the original version
was also started when I had a very different skill level and different amount of
knowledge.

The original Invader also has a lot of warts due to features being tacked on, as
the original implementation of Invader had no real goal except to compile
multiplayer Halo Custom Edition cache files.

It was not anticipated that it would become a *complete* toolkit used for making
assets from scratch, as it was not fully realized how important this need truly
was at the time Invader was started.


## Issues that are solved by Invader

At the time, it was widely considered that most of our needs were met by what we
had:
- We have a tag editor in the HEK's Guerilla and the MEK's Mozzarilla
- We have a tag extractor in the MEK's Refinery
- We can build cache files and make bitmaps, sounds, models, etc. with tool.exe
- We have a scenario editor that implements the entire game with Sapien

Having something that built cache files a little better was more of a fun side
project at first - an open source application to go along with the MEK to help
out with making a decent open source toolkit.

It turns out, however, that Halo Custom Edition and Halo CEA's editing kits have
a plethora of issues that can ONLY be solved with a *complete* open source
reimplementation.

- **Poor error checking:** The original HEKs have some error checking, but they
  often refers to source files that are not publicly available, and any messages
  that are printed are quite vague. Often it's just assertions or, at worst,
  exception errors that lack any useful information at all. And in most cases,
  it will not even error on actual problematic tag data, leaving it up to you,
  the modder, to find out if something will crash the game or not.

  When it comes to making a game, this is probably fine. Bungie likely did not
  expect this editing kit to be used outside of being used internally within the
  studio to make Halo (or Stubbs the Zombie that one time). The error checking
  that is there was enough to make the game. However, modders need a bit more to
  make custom content, as the whole engine is basically a black box due to being
  closed source.

  Having good quality, accurate errors is necessary, and it is what Invader does
  extremely well even on the original C++ version (though a rewrite can maybe do
  even better!).

- **Lack of information:** The tools being closed source with almost no official
  documentation means that it can be difficult to understand the tag system and
  other aspects of editing Halo content.

  Invader is completely free and has been since its very first public release.
  By having a complete open source implementation of a modding toolkit, we can
  better understand Halo's engine as a whole, as all of the verification and
  experimentation with learning how this engine works has already been done and
  is documented via Invader's source code. Having public information and more
  allows the community to be sustainable.

  Thanks to Invader being a free toolkit, we now account for every byte that is
  stored in a cache file - every last bit that is loaded into the game engine.
  Every hidden field is known, and things that are preprocessed are documented.
  There are no longer any "gotchas" when it comes to modding the game thanks to
  free tools like Invader.

- **Performance issues:** It takes an incredible amount of time to build cache
  files or open large tags with the Halo Editing Kits. Opening BSP tags with
  complex geometry can take over 20 seconds on modern PCs, even those built
  after the year 2016 (over 15 years after the original Xbox release of Halo).
  This may not seem like a long time, but when you are constantly opening and
  closing tags, 20 seconds adds up to a lot. Current Invader takes less than one
  second to open these same tags on the same machines, and it can even compile a
  map faster than it takes for Guerilla to open one of that map's BSPs.

  The original tools are compiled for 32-bit x86 on Windows even though the
  current release of Halo: Combat Evolved Anniversary natively supports modern,
  64-bit x86 Windows.

  Allegedly, the many checks and assertions done with the game slow it down
  further. The current C++ implementation of Invader demonstrates that we can
  have powerful error checking and even more functionality while achieving even
  higher performance than the original Halo Editing Kits. We can even target
  modern PCs and Linux which Invader does directly support via an
  [Arch Linux AUR package].

  [Arch Linux AUR package]: https://aur.archlinux.org/packages/invader-git


## What's wrong with the current C++ Invader?

As mentioned before, Invader had an issue where things were tacked on for the
sake of implementing a feature needed at the time.

For example, the Python code is a horribly unreadable mess that generates a lot
of C++ code to make actually running Invader fast. This results in long compile
times, and it's very hard to incrementally work on the parser.

Also, there exists a `hek` module that was originally there for definitions that
were reverse engineered or extracted from the Halo Editing Kit. This has become
an inconsistent, confusing mess that has resulted in things taking a while to
find around the source tree even for myself.

I also think a lot of code can be done better. Most of the existing code is fine
to use, but a lot of it was written in a bit of a hacky way to "get things done"
when it came to targeting multiple engines. Definitions, for example, lack any
sort of flags to distinguish between different engines of the game. These are
things that should have been added in on the start for such a project, but once
again, I did not foresee Invader becoming as big as it did.


## Why Rust?

Rust is a good language with a strong emphasis on safety without compromising on
speed. This is great for sustainability with extremely few developers working on
such a big project. It is frustrating to unknowingly break something that has
been in the code for years or to trigger a hard-to-debug segmentation fault.

The Rust ecosystem also provides built in functionality for unit tests and
external modules through the `cargo` program. This means that code can be tested
for breakage before being released.

Static linking is also much easier to do with Rust as it is done by default.
Dynamic linking is an antipattern that results in distributing tons of DLL files
and also executables that do not last very long. Dynamic linking makes sense
only for system libraries like kernel.dll or the Linux kernel, as these are APIs
that are stable and are guaranteed to be present on the target machines. Dynamic
linking libraries not expected to be on the target machine results in dependency
hell. For Linux, package managers try to solve this, but API breakage happens.
For Windows, distributing over a hundred MB of DLLs with a toolkit leads to a
poor experience using the tools, and there are some cases where DLLs may
conflict or need to be updated individually.

All-in-all, Rust and Cargo provide the right tool for the job. I still think C++
is a fantastic language, but Rust, itself, is designed to solve many problems I
have had when working on Invader.


## What will happen to the C++ project?

For now, the C++ project will continue to be maintained while the Rust rewrite
is not yet at feature parity. Support and updates for future versions of Halo:
Combat Evolved Anniversary's modding features will continue to take place, and
major bugs will continue to be fixed.

However, once the rewrite contains all features from the C++ version of Invader
and proves to be a viable replacement, the C++ version will be fully dropped.
Also, the rewrite's version number will continue off from the C++ version number
at whatever it is at the time of being finished (e.g. if Invader is at version
0.60.3, then the rewrite will be 0.61.0). Until then, the Rust rewrite's version
version number will stay at 0.1.0 as a placeholder.
