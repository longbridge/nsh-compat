# nsh-compat

The API [Native Shell](https://github.com/longbridge/nsh)'s releases froze, and the checks that hold later
checkouts of NSH to it.

NSH apps are delivered to host apps as plugins and may never be built again, so every release of NSH runs every app
built for an earlier compatible one. What a release shipped is recorded here, apart from NSH's code, so a change to
NSH cannot rename an API and rewrite the record of it in the same pull request. The contract itself is
[NSH's docs/dev/compatibility.md](https://github.com/longbridge/nsh/blob/main/docs/dev/compatibility.md).

```
releases/v<version>/    one per compatible release, written when it ships and never changed:
  runtime.txt           every module's exports (`nsh:rt` included) and the globals
  types/                the declarations apps build against
  tailwind.txt          every Tailwind class and what it sets
  apps/<app>/           the apps of NSH's tests/compat/ as that release built them, with their scripts
closed.txt              what was closed for security, the only removals, each with its reason
src/                    the checks
```

No release is recorded yet: nothing is frozen.

## Running it

The crates depend on NSH's at `../nsh`, the checkout under test. From an NSH checkout:

```sh
cargo xtask compat                      # clones this into target/nsh-compat/ and runs it against the checkout
cargo xtask compat record <version>     # records the checkout as a release here, to commit
```

With both repositories side by side:

```sh
(cd ../nsh && cargo build -p nsh-cli)
cp ../nsh/Cargo.lock . && cargo test --target-dir ../nsh/target
```

The tests:

- `checkout_apps_run`: NSH's `tests/compat/` apps, built by the checkout's `nsh build --release`, do what their
  scripts say, and their `covers` claims hold;
- `nothing_released_is_removed`: everything each release here offered is still offered, the same or widened, less
  `closed.txt`;
- `released_apps_run`: every app each release built still starts and does what its script says.

`NSH_COMPAT_APP=<name>` selects apps whose unqualified name starts with `<name>`; `desktop` selects both its case
and example. `NSH_COMPAT_DUMP=1` prints the texts each step of a script adds and removes,
`NSH_COMPAT_KEEP_GOING=1` goes on after a failed step at the script's next case (its next `tap id:case-…`) and reports
the first failure of each case at the end, so one run shows every case that fails.

Cases and examples build concurrently into separate `case-<name>` and `example-<name>` directories. Each owns
its `.build` staging directory, so equally named apps cannot overwrite each other's bundle, metadata or script.
Each checkout run replaces its generated output directory before building; frozen release records are unchanged.

Material assertions use `prop <selector> macos.backdrop material=hudWindow blending=behindWindow`
and `style <selector> glassTintColor #ff000080`. Backdrop members at their defaults are omitted;
an explicitly requested default material is `{}`, while a removed material has no `backdrop` key.
The build probes NSH's public prop/style APIs before compiling their field formatters, so this checker
also builds against otherwise supported NSH checkouts that predate macOS backdrops and glass tint.
Material tests initialize `MacOsBackdrop` from `Default` before assigning fields, so they also compile
when NSH marks that public configuration `#[non_exhaustive]`.

## Changing it

A pull request here is reviewed by NSH's maintainers. A release record is added once, at the release, and never
changed. A closure for security adds its entries to `closed.txt` with a reason, before the NSH change that removes
them can pass. Changes to the checks keep every recorded release passing; CI runs them against NSH's `main`.
