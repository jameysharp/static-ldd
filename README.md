static-ldd
==========

This is a library and command-line tool for inferring dependencies
between static libraries.

Each dynamic library (also known as a shared object) on Unix-like
systems includes machine-readable information about other libraries that
you also need to link against if you want to use it, and the dynamic
linker automatically loads those dependencies, recursively, as needed.

Since modern software engineering encourages small, well-encapsulated,
reusable software, applications generally have transitive dependencies
on at least a few dozen libraries, and sometimes more. If the
application developer had to identify every library their software had a
transitive dependency on, it would be a nightmare to build anything new.
So this automatic dependency tracking behavior for dynamic libraries is
super helpful.

Unfortunately, static libraries do not carry library dependency
information, so linking against static libraries is more troublesome.
Making matters worse, the order in which the libraries are listed to the
linker is important, because during static linking each library is
visited only once, and the linker may not find the symbols one library
needs if the library that provides them is listed in the wrong place.

Many projects provide
[`pkg-config`](https://www.freedesktop.org/wiki/Software/pkg-config/)
metadata as an external record of dependencies, which is an excellent
solution and should be your first choice when possible. However, not all
do.

When `pkg-config` metadata is not available, you can use `static-ldd` to
infer the dependencies within a group of static libraries, and determine
a correct link order that respects those dependencies.

Usage
-----

From the command line:

```
static-ldd <roots>... -- <others>...
```

`<roots>` are the libraries or object files that contain code you know
you need. This could be your object files produced by compiling your
project's source code.

`<others>` are the libraries that you think you might need. If there's
no transitive dependency from a root to one of these libraries, it won't
be included in the final link list.

The output of the command-line tool is currently only really suitable
for human consumption (and hardly even that). I'd love to receive
patches making the command-line tool usable in build systems.

Using this crate as a Rust library involves first calling
`dependency_map` with an iterable listing all the libraries, both roots
and others, to get a map of all the dependencies; and then passing that
map and the iterable list of roots to `needed_libraries` to get the
final link list.

Eventually there should be a helper function for use in Cargo build
scripts that will automatically add all the discovered libraries to the
linker flags, but that isn't implemented yet.

License
-------

This crate is dual-licensed Apache-2.0/MIT, which probably allows any
use you care about. However, you should verify that your use is
compatible with the licenses of this crate's dependencies. In
particular, [`filebuffer`](https://crates.io/crates/filebuffer) is (as
of this writing) licensed only under Apache-2.0, which might create
issues for you if you want to use GPLv2-licensed code with it. This is
not legal advice; if in doubt, consult a lawyer.
