# Releasing Nitrogen

```
cargo release <VERSION LEVEL> --execute --no-publish
```

Where `<VERSION LEVEL>` is one of `major`, `minor`, or `patch` depending on what version of nitrogen you want to release.

Run the following to actually upload to cargo:

```
cargo publish
```

Next you need to manually make the release in github from the tag. This will kick off the building process
to build all the releases assets and store them on the release in github.

Afterwards we need to publish an alpha version to prepare for the next release.

```
cargo release alpha --execute --no-publish
```
