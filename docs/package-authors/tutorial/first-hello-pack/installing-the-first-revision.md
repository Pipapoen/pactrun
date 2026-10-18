---
title: Installing the First Revision
---

# Installing the First Revision

You have a Revision Pack source in `pack/`, but Pactrun has not installed it yet.
Installation checks the source and saves its management definition and runtime
content as an immutable **Revision** in the default Store. It does not create
an Instance or run a program.

## Install the Revision Pack source {#install-the-pack-source}

From your `hello-pack-tutorial` working directory, make sure you replaced the
placeholder Package ID in `pack/pactrun.yaml` with the ID you generated. Then run:

```sh
pactrun revision install ./pack
```

Pactrun reads `pack/pactrun.yaml` and validates the source. For example, a
successful first installation looks like this:

```text
Revision installed.

Package:Revision: 0123456789ab:abcdef012345
Installed: 2026-10-17T08:00:00Z
```

The IDs and time are examples; your values will differ. Keep the `Package:Revision:`
reference for the next step. If installation fails, read the
diagnostic and correct the source before continuing; a failed installation has
not given you a usable Revision.

The source directory remains yours to edit. Installing it saves these exact
contents; later edits to `pack/` do not change the installed Revision.

## Find the installed Revision

Find your Package in the default Store, then list its Revisions. Replace
`PACKAGE_ID` with the ID from `pactrun.yaml` or the Package list:

```sh
pactrun package list
pactrun revision list 'PACKAGE_ID'
```

If this is the only Revision in your Store, the result looks like this:

```text
PACKAGE:REVISION            INSTALLED (UTC)       SUMMARY
0123456789ab:abcdef012345  2026-10-17T08:00:00Z

1 record shown.
```

The Revision list contains only this Package's versions, newest-installed first.
The left part of each entry identifies its Package; the right part identifies
its Revision. Names may appear when assigned; otherwise unique ID prefixes are
shown. Use `--no-trunc` to see full IDs. This query does not create an Instance.

Now inspect the Revision you just installed. Replace `REVISION_REFERENCE` with
the Package:Revision value printed by `revision install`:

```sh
pactrun revision show 'REVISION_REFERENCE'
```

Using the example reference above, the result looks like this:

```text
Package:Revision: 0123456789ab:abcdef012345
Installed: 2026-10-17T08:00:00Z
Core format: 1.0-alpha.2
Package ID: 0123456789abcdef0123456789abcdef
Revision ID: abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789

No Actions, Inputs or managed capabilities declared.
```

The `Package ID:` and `Revision ID:` lines show the full IDs. Because
`revision: {}` contains no operations or Inputs, there are no capabilities to
list.

The `revision:` field in `pactrun.yaml` holds the version's declarations,
including `runtime_content`. The Revision ID is computed from their normalized
meaning and the acquired files. `Core format: 1.0-alpha.2` names the installed
definition's encoding, while `source_format: "1.0-alpha.4"` selects the YAML
authoring schema. Neither format number is a product version or a Revision ID.

## Install the same contents again

Without changing anything in `pack/`, run:

```sh
pactrun revision install ./pack
```

The output should now begin with `Revision already installed.` and show the
same Revision reference and installation time:

```text
Revision already installed.

Package:Revision: 0123456789ab:abcdef012345
Installed: 2026-10-17T08:00:00Z
```

Repeating installation does not create a new Revision when the
operational contents are unchanged.

You now have a valid installed Revision, but it has no Action to invoke. Next, you
will add the first operation and its implementation before creating an Instance.
