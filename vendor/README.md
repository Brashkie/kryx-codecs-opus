# vendor/

Third-party vendored libraries.

## libopus 1.5.2 (BSD-3-Clause)

- **Source**: https://github.com/xiph/opus
- **Release**: https://github.com/xiph/opus/releases/tag/v1.5.2
- **License**: BSD-3-Clause (see `libopus/COPYING`)
- **Copyright**: 2001-2023 Xiph.Org, Skype Limited, Octasic, Jean-Marc Valin,
  Timothy B. Terriberry, CSIRO, Gregory Maxwell, Mark Borgerding,
  Erik de Castro Lopo, Mozilla, Amazon

### Why vendored?

- **Reproducibility**: every build uses exactly libopus 1.5.2
- **No network during build**: `npm install` works offline
- **No system dependency**: users don't need libopus installed on the host

### Modifications

The libopus sources are vendored unmodified, except that the following
non-runtime directories were removed from the v1.5.2 tarball to reduce
repo size:

  - `dnn/torch/` — PyTorch training scripts (~10 MB)
  - `dnn/training_tf2/` — TensorFlow training scripts
  - `doc/` — HTML documentation (regenerable from source)
  - `tests/` — libopus internal test suite
  - `training/` — Training utilities

All runtime sources are preserved (`celt/`, `silk/`, `src/`, `include/`,
`dnn/` runtime `.c`/`.h`, `cmake/`, `m4/`, `meson/`, `scripts/`).

### How to update libopus

When a new libopus version is released and we want to update:

```bash
# 1. Download the new tarball
curl -sL https://github.com/xiph/opus/releases/download/v1.X.Y/opus-1.X.Y.tar.gz -o /tmp/opus.tar.gz

# 2. Replace vendor/libopus/
rm -rf vendor/libopus
mkdir vendor/libopus
tar -xzf /tmp/opus.tar.gz --strip-components=1 -C vendor/libopus

# 3. Remove non-runtime dirs
cd vendor/libopus
rm -rf dnn/torch dnn/training_tf2 doc tests training

# 4. Update NOTICE and CHANGELOG with the new version
# 5. Verify build still works
```
