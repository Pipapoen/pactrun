---
title: Vocabulary
---

# Vocabulary

| Term | Meaning | Details |
| --- | --- | --- |
| Package | Stable service-management product lineage, identified by a Package ID | [System model](./system-model.md), [identity](../packages/identity.md) |
| Revision Pack | Installable bundle of one Revision's definition and files, Package ID and optional descriptions | [Revision Packs](../packages/revision-packs.md) |
| Revision | Exact immutable operational definition and owned runtime content | [Object model](./objects.md), [Core format](../packages/revision-format.md) |
| Revision ID | Content digest identifying a version within its Package; 64 lowercase hexadecimal digits in current public interfaces | [Identity](../packages/identity.md) |
| Local name | User-assigned Package or Revision reference within a Store | [Local names and resolution](../packages/local-names.md) |
| Instance | Long-lived managed object with stable identity and an active Revision | [Instance identity and state](../instances/identity-state.md) |
| Managed Input Binding | Detached value whose authoritative bytes are owned by Pactrun | [Bindings and protection](../instances/bindings.md), [Input behavior](../instances/inputs-secrets.md) |
| Secret | Protection and disclosure rules, not a claim that Pactrun is a secret vault | [System model](./system-model.md), [Input behavior](../instances/inputs-secrets.md) |
| ServiceStorage | Instance-scoped persistent allocation supplied by Pactrun for service or Hook use | [ServiceStorage execution](../instances/service-storage.md) |
| Managed Service Resource | Separately identified service-owned file or directory declared within a ServiceStorage allocation | [Resource ownership](../instances/service-resources.md) |
| Workspace | Execution-scoped scratch under granted Hook authority | [Hook authority](../interfaces/hooks.md) |
| Plan | Compiled intent and checks, not a durable replay log | [Execution](../operations/execution.md) |
| Run | Durable execution-attempt record, distinct from an ephemeral Plan | [Execution](../operations/execution.md), [behavior](../operations/actions-plans-runs.md) |
| Hook | Trusted service-specific code operating through the defined context and authority | [Hook semantics](../interfaces/hooks.md), [wire](../interfaces/hook-protocol.md) |
| Snapshot | Immutable recovery representation with its own identity and lifetime | [Snapshot contracts](../snapshots/definition.md) |
| Run Artifact | Managed output with retention separate from Workspace and Snapshot lifetimes | [Resources](../lifecycle/resource-lifetimes.md) |
| Migration | Target-owned Revision transition; Managed Input transitions are not atomic service-database migration | [Migration contract](../migrations/definition.md) |
| Recovery / reconciliation | Decisions based on durable ownership and risk evidence, not automatic Hook replay | [Recovery](../lifecycle/recovery.md) |
