---
title: User vocabulary
---

# User vocabulary

| Term | What it means when operating Pactrun |
| --- | --- |
| Package | A service-management product lineage with a stable Package ID, whose versions are Revisions. |
| Revision Pack | A bundle you install or share, containing one version's definition and files. Run only content whose Hooks you trust. |
| Revision | One exact immutable installed definition and its runtime files. Installing another Revision does not switch an existing Instance. |
| Revision ID | A content-derived 64-digit lowercase hexadecimal ID identifying the version within its Package. |
| Instance | A long-lived managed object with a name, identity, active Revision, configuration, and recovery state. |
| Input | A detached, Pactrun-owned copy of opaque configuration bytes. Changing the original file does not update the binding. |
| Required Input | An Input needed by a particular operation. With current-format Revisions, removing its binding makes that operation's requirements unsatisfied. Legacy Core `1.0-alpha.1` instead has global required declarations and prevents ordinary removal of an active required binding once bound. |
| Secret | Protected handling/disclosure rules. Not an encrypted vault, sandbox, or guarantee that a trusted Hook cannot disclose it. |
| Parameter | A typed value for one invocation. It is not persistent Instance configuration. |
| Action | An operation declared by a Revision; inspect its prerequisites and effects before running it. |
| Hook | Trusted host code that implements an operation. Names such as inspect or backup do not prove what its code does. |
| Plan | Read-only preview of an operation. It reserves nothing and does not guarantee later execution will be accepted. |
| Run | Durable record of an accepted execution attempt. Progress or child-process exit alone does not prove a successful terminal outcome. |
| State-version token | Exact observed Instance state used for conditional writes. Do not abbreviate it or automatically replace a stale expectation. |
| Store | Pactrun's management data location for installed versions, Instances and retained records. |
| ServiceStorage | Persistent storage allocated for one Instance and made available to its service or Hooks. |
| Managed Service Resource | A separately declared service-owned live file or directory within ServiceStorage. Declaring it does not create it or automatically back it up. |
| Workspace | Execution-scoped scratch. Do not keep its path as durable service state. |
| Artifact | Explicitly registered managed output retained separately from Workspace; export before deliberate deletion. |
| Snapshot | Immutable recovery representation with complete managed bindings and authored service content. Not every host resource or executable Pack is included. |
| Migration | A declared path to another Revision in the same Package. Each edge commits separately; there is no whole-path rollback promise. |
| Retained binding/resource | State preserved after it stops being active. Inspect its role and the allowed operation rather than treating it as current configuration. |
| Manual recovery | Pactrun cannot establish the required service consistency. Inspect and repair externally before explicitly acknowledging it. |
| Delete with Cleanup | Run the authored retirement procedure and authorized finalization; failure does not imply the Instance or data is gone. |
| Abandon | End Pactrun management while preserving remaining service data. It does not stop the service. |
| Detached allocation | Retained service storage after management ends. Ordinary GC does not discard it. |
| GC | Foreground removal of eligible unreferenced managed content. It does not replace Instance retirement or service-data disposal. |

For procedures, return to [user tasks](../../guides/index.md). For complete
options and recovery conditions, use the [user reference](./index.md).
