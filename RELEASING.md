# Releasing Nitrogen

```
cargo release <VERSION LEVEL> --execute --no-publish
```

Where `<VERSION LEVEL>` is one of `major`, `minor`, or `patch` depending on what version of nitrogen you want to release.

Run the following to actually upload to cargo:

```
cargo publish
```

Afterwards we need to publish an alpha version to prepare for the next release.

```
cargo release alpha --execute --no-publish
```
