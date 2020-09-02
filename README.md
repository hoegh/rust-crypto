# Quick intro

SHA algorithms implemented in Rust. Build with Cargo (for release, if you need decent performance).

Currently the following algorithms are implemented:
* SHA-256
* SHA-512

There are two "main" for running the code, one for each of the two algorithms - see src/bin.

If not given any parameters, they will scan the current directory for files and calculate a hash for each.

If given parameters, it is expected to be a list of files. For each that cannot be read as file, a message will be emitted to stderr - hashes calculated for the rest.

# Notes

This is *not* an implementation that is meant for production use. It is written as a learning exercise for me, and the SHA algorithms fit my purpose.

Neither does it need to be safe. But it is tested against a set of official test-vectors.

It is always hard to measure performance, but is seems to be decent, compared to the native (linux) sha256sum resp. sha512sum on my machine.

I set out to explore iterators in Rust, but ended up bypassing them for performance reasons; files should be read with a decent buffer, and memory should not be copied unnecessary; having a stream of bytes may be convenient, but it surprisingly this ends up being quite significant compared to calculating the actual hash. Though the iterator based code is still there, as seen from the mains perspective, it is dead code (but still fully functional).
