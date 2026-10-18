---
title: Commands and output
---

# Commands and output

Use `pactrun --help` to inspect the executable you are running. The website's
**Command help** page is generated from the edition's CLI help catalog and
identifies its product version. Select the documentation edition matching your
executable when following examples.

Help follows the command hierarchy. For example, `pactrun instance --help`
lists Instance operations, while `pactrun instance create --help` shows creation
forms and options. Help paths need no operation operands or Store. The same
queries work with `--format json` or `--format jsonl` before the command; machine
responses include a structured catalog for the requested scope.

For Action parameter syntax, timeouts, defaults, and examples, read the
[Invoke command reference](./invoke-reference.md), or run `pactrun invoke --help`.

## Choose a command family

| Task | Commands | Reference |
| --- | --- | --- |
| Install or export a Revision | `revision` | [Command details](../reference/command-details.md) |
| Remove a Package or one Revision | `package delete`, `revision delete` | [Retained objects and GC](../reference/object-lifecycle.md) |
| Create and inspect Instances | instance create/list/show | [Command details](../reference/command-details.md) |
| Supply or export configuration | input | [Command details](../reference/command-details.md) |
| Execute and inspect operations | action, invoke, run | [Invoke](./invoke-reference.md) |
| Inspect live data declarations | service-storage, resource | [Resources](../reference/resource-details.md) |
| Back up or transition | snapshot, instance migrate | [Snapshots](../reference/snapshot-details.md), [Migration](../reference/migration-details.md) |
| Retire and reclaim | instance delete/abandon, storage gc | [Retirement](../reference/retirement-details.md) |

## References and pagination

Revision references use `<package>:<revision>`. Either component may be a user
name or an ID, so `app:stable`, `<package-id>:stable`, `app:<revision-id>` and
`<package-id>:<revision-id>` are supported. Names are local, case-sensitive and
1 to 31 ASCII characters. Unique lowercase hexadecimal ID prefixes require at
least eight digits. Full IDs remain usable even when names or short prefixes are
ambiguous. A Revision ID is the 64-digit content digest within its Package,
without `sha256:`. Use `package list` to discover lineages, `package show` to
inspect one, and `revision list <package>` to list its versions. `revision install`
reads a Revision Pack; `package generate-id` only generates a lineage ID.
State-version tokens and authored capability identifiers still require exact values.

Where supported, lists use `--limit`, `--after`, and `--no-trunc`. Follow the
reported continuation rather than assuming the first page is complete. Pages
are separate read snapshots, not a frozen export of the whole catalog.
Revision lists are newest-installed first, with unknown times last; their returned
cursor includes its sort position and is not an editable Revision reference.

## Remove a version or a product lineage

Use `revision delete <package>:<revision>` to remove one installed version.
The Package and its local name remain even when no Revisions are left.

Use `package delete <package>` to remove a Package, its local name and every
installed Revision. You can select the Package by its complete ID, a unique ID
prefix or a local name. Pactrun checks all its Revisions first: an Instance using
any version, a retained Run reference, service association or recovery requirement
blocks the whole deletion. A refused deletion leaves all versions in place.

Package deletion does not retire Instances, run Hooks or remove service-owned
data. Independent Run history and Snapshots remain. Shared physical content is
reclaimed through normal garbage collection. Installing a Revision Pack with the
same Package ID later creates the Package again, without its former local name.
Restoring a retained Snapshot still requires its exact producer Revision; keep
the matching Revision Pack for reinstallation because Snapshot bundles do not
contain its executable runtime content.

## Output modes

- Human output is for reading. Do not parse its tables as a stable API.
- `--format json` emits a complete response.
- `--format jsonl` streams events followed by a terminal result.

Check the response contract, result, and process exit status. Hook byte chunks
in machine modes use Base64. Cancellation on a closed JSONL receiver is opt-in
with `--cancel-on-output-close`; a disconnected consumer must not assume the
managed operation stopped by default.

## Plans, timeouts, and errors

Only use `--plan` on commands that support it. Plans do not reserve admission.
Snapshot and Migration timeout options differ from invoke's Action timeout;
use the [Snapshot options](../reference/snapshot-details.md) and
[Migration options](../reference/migration-details.md) rather than copying flags.
Syntax errors and operation failures have different exit conventions. A partially
completed operation needs inspection even when its final exit is nonzero.

More detail: [User reference](../reference/index.md).

<details>
<summary>Maintainer sources (optional)</summary>

Contracts: [commands](../../spec/interfaces/commands.md),
[machine output](../../spec/interfaces/machine-output.md), and
[selectors](../../spec/interfaces/id-selectors.md).

</details>
