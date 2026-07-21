# Third-Party Notices

This repository pins the official Google LiquidFun source at
`7f20402173fd143a3988c921bc384459c6a858f2` as a read-only, development-only
oracle. [`reference/upstream-lock.toml`](reference/upstream-lock.toml) records
the full identity.

The root [`LICENSE`](LICENSE) applies to original `liquidfun-rs` work. It does
not replace the licenses, notices, source-origin disclosures, or altered-source
duties that apply to upstream or derived material. This notice inventory is a
compliance record, not a final legal conclusion about future translated work.

## LiquidFun and Box2D

The pinned authoritative files are:

- `third_party/liquidfun/liquidfun/Box2D/License.txt`
- `third_party/liquidfun/liquidfun/NOTICE`

### Box2D/LiquidFun source license

```text
Copyright (c) 2006-2013 Erin Catto http://www.gphysics.com

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
claim that you wrote the original software. If you use this software
in a product, an acknowledgment in the product documentation would be
appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.
```

### LiquidFun notice

```text
Copyright (c) 2006-2010 Erin Catto http://www.gphysics.com

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
claim that you wrote the original software. If you use this software
in a product, an acknowledgment in the product documentation would be
appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.
```

## Developer-Only Dependencies

### nlohmann/json

`tools/reference/vendor/nlohmann/json.hpp` is the official single-header
release artifact for nlohmann/json 3.12.0. It is vendored solely for the
private C++ oracle adapter and is not a runtime, build-time download, or
dependency of any published Rust crate or ordinary Cargo consumer path.

The component is licensed under the MIT License. Its complete, verbatim
license and copyright notice are preserved in
`tools/reference/vendor/nlohmann/LICENSE.MIT`. The reviewed version, official
release and tag-pinned source URLs, and repository-local SHA-256 checksums are
recorded in `tools/reference/vendor/nlohmann/SHA256SUMS`.

### Pinned upstream test and testbed dependencies

The pinned upstream tree vendors dependencies used by its tests and visual
testbed:

| Component  | Pinned license file                                                                       | Scope                                              |
| ---------- | ----------------------------------------------------------------------------------------- | -------------------------------------------------- |
| GoogleTest | `third_party/liquidfun/googletest/LICENSE` and `third_party/liquidfun/googletest/COPYING` | BSD-3-Clause-style terms; upstream unit tests only |
| freeglut   | `third_party/liquidfun/freeglut/COPYING`                                                  | MIT-style terms; upstream visual testbed only      |

These trees remain under the development-only submodule. They must not be
copied into, linked by, or included in the published Cargo package. Package
isolation checks must fail if `third_party/liquidfun`, GoogleTest, freeglut, or
their assets enter the consumer archive.

## Derived and Altered Material

Every local source file, translated test, scenario, fixture, reference datum,
or other artifact derived from the upstream tree must have an entry in
[`reference/source-map.toml`](reference/source-map.toml). The entry must record
its local path, full upstream revision and path, derivation kind, alteration
summary, and notice class.

Altered or translated source must be plainly identified as altered, preserve
applicable notice text, and never be represented as original project work.
Generated reference artifacts additionally require reproducible content and
generator hashes when their manifest schema is introduced. Unmapped derived
material or an unresolved notice classification blocks packaging and release.
