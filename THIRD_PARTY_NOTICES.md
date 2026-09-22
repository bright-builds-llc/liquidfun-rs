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

## Web playground runtime dependency

The private unpublished `liquidfun-web` package (`web/`) pins `@kobalte/core`
0.13.12 as a production dependency for accessible dialog behavior in the
responsive playground shell. Official source:
<https://github.com/kobaltedev/kobalte>. Ordinary Rust crate consumers do not
receive these npm packages; they resolve only through the web playground build
and are not part of the published `liquidfun` Cargo package.

Preserved upstream notice and license text (byte copies of the installed npm
artifact after `bun install --frozen-lockfile`):

- [`web/licenses/kobalte-core-0.13.12-NOTICE.txt`](web/licenses/kobalte-core-0.13.12-NOTICE.txt)
- [`web/licenses/kobalte-core-0.13.12-MIT.txt`](web/licenses/kobalte-core-0.13.12-MIT.txt)
- [`web/licenses/kobalte-core-0.13.12-MIT.provenance.md`](web/licenses/kobalte-core-0.13.12-MIT.provenance.md)

The locked runtime closure reachable from `@kobalte/core@0.13.12` in
`web/bun.lock` is inventoried in
[`web/licenses/kobalte-runtime-closure.json`](web/licenses/kobalte-runtime-closure.json).
`solid-js` remains a separate direct production pin (peer of Kobalte) and is not
part of that closure list. **Not every transitive package is MIT**; the
non-MIT SPDX identifiers in this closure are Apache-2.0 and 0BSD:

| Package                             | Locked version | SPDX license | Official source                                         |
| ----------------------------------- | -------------- | ------------ | ------------------------------------------------------- |
| `@corvu/utils`                      | 0.4.2          | MIT          | <https://github.com/corvudev/corvu>                     |
| `@floating-ui/core`                 | 1.8.0          | MIT          | <https://github.com/floating-ui/floating-ui>            |
| `@floating-ui/dom`                  | 1.8.0          | MIT          | <https://github.com/floating-ui/floating-ui>            |
| `@floating-ui/utils`                | 0.2.12         | MIT          | <https://github.com/floating-ui/floating-ui>            |
| `@internationalized/number`         | 3.6.8          | Apache-2.0   | <https://github.com/adobe/react-spectrum>               |
| `@kobalte/core`                     | 0.13.12        | MIT          | <https://github.com/kobaltedev/kobalte>                 |
| `@kobalte/utils`                    | 0.9.2          | MIT          | <https://github.com/kobaltedev/kobalte>                 |
| `@solid-primitives/event-listener`  | 2.4.6          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/keyed`           | 1.5.3          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/map`             | 0.4.13         | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/media`           | 2.3.6          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/props`           | 3.2.4          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/refs`            | 1.1.4          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/resize-observer` | 2.2.0          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/rootless`        | 1.5.4          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/static-store`    | 0.1.4          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/trigger`         | 1.2.4          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@solid-primitives/utils`           | 6.4.1          | MIT          | <https://github.com/solidjs-community/solid-primitives> |
| `@swc/helpers`                      | 0.5.23         | Apache-2.0   | <https://github.com/swc-project/swc>                    |
| `solid-presence`                    | 0.1.8          | MIT          | <https://github.com/corvudev/corvu>                     |
| `solid-prevent-scroll`              | 0.1.11         | MIT          | <https://github.com/corvudev/corvu>                     |
| `tslib`                             | 2.8.1          | 0BSD         | <https://github.com/Microsoft/tslib>                    |

Repository check: `cd web && bun run verify:kobalte-licenses` recursively derives
the runtime dependency closure from package metadata installed by
`bun install --frozen-lockfile`, requires the tracked inventory to match it
exactly with no missing or extra entries, validates version, license, and source
metadata, and verifies the preserved NOTICE and MIT files remain byte-identical
to the installed `@kobalte/core` artifact.

The playground shell also vendors UI adapted from
[shadcn-solid](https://github.com/hngngn/shadcn-solid) (MIT) and depends on these
direct packages. They stay inside the private `web/` build and are not part of
the published `liquidfun` Cargo package.

| Package             | Locked version | SPDX license | Official source                                 |
| ------------------- | -------------- | ------------ | ----------------------------------------------- |
| `@corvu/drawer`     | 0.2.4          | MIT          | <https://github.com/corvudev/corvu>             |
| `@tailwindcss/vite` | 4.3.3          | MIT          | <https://github.com/tailwindlabs/tailwindcss>   |
| `cva`               | 1.0.0-beta.4   | Apache-2.0   | <https://github.com/joe-bell/cva>               |
| `tailwind-merge`    | 3.7.0          | MIT          | <https://github.com/dcastil/tailwind-merge>     |
| `tailwindcss`       | 4.3.3          | MIT          | <https://github.com/tailwindlabs/tailwindcss>   |
| `tw-animate-css`    | 1.4.0          | MIT          | <https://github.com/Wombosvideo/tw-animate-css> |

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
