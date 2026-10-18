---
title: Package and Revision Identity
---

# Package and Revision Identity

A Package ID identifies a service-management product lineage. A Revision is one
immutable version of its management definition and owned runtime content. Its
Revision ID is the content digest, rendered as 64 lowercase hexadecimal digits
within its Package. The full management key combines Package ID and Revision ID.
Presentation, provenance, and local trust do not change either ID.

### PR-REQ-0011 - Stable Package lineage

`PackageId` MUST be opaque, stable, and independent of content. It MUST survive
Revision changes, presentation or publisher changes, export, and import. A fork
that becomes a new lineage MUST be explicitly re-identified.

The Revision Pack source declares this identity in the required exact
`package_id` in `PackSourceYamlV1`; installation MUST NOT derive a lineage from
source location or content. ID generation is a separate non-mutating operation.

**Verification: PR-TEST-0067, PR-TEST-0069, PR-TEST-0142, PR-TEST-0594, PR-TEST-0600.**

### PR-REQ-0012 - Operational content digest

`RevisionContentDigest` MUST identify Pactrun-managed operational semantics and
immutable owned runtime content. It MUST exclude `PackageId`, display content,
publisher claims, source location, install time, local names, notes,
and local trust decisions. ServiceStorage and Managed Service Resource
declarations are identity-bearing under the
[Revision baseline](./revision-format.md). The service-owned live
bytes MUST remain outside the Revision digest.

**Verification: PR-TEST-0044, PR-TEST-0067, PR-TEST-0331, PR-TEST-0346, PR-TEST-0493, PR-TEST-0594, PR-TEST-0600.**

### PR-REQ-0013 - Exact Revision identity

A Revision ID MUST be the existing content digest within one Package. The
internal `RevisionIdentity` management key MUST retain both `PackageId` and
`RevisionContentDigest`. Equal Revision IDs under different Package IDs MUST
remain separately managed versions; naming the digest a Revision ID does not
make it a globally unique Package-independent object key.

**Verification: PR-TEST-0053, PR-TEST-0054, PR-TEST-0594.**

### PR-REQ-0014 - RevisionCore projection

`RevisionCore` MUST be a versioned, identity-bearing semantic projection of a
validated `NormalizedPackDefinition`. It MUST NOT be defined by serializing a
Rust type, database row, authoring AST, or the entire normalized authoring
model.

The projection MUST include every field that can affect invocation semantics,
parameter behavior, compiler or executor behavior, operation access, Hook
Session authority, Inputs, Snapshot, Migration, Cleanup, failures, recovery, or
service-facing runtime behavior. Presentation, attribution, provenance, and
local-management data MUST remain outside it.

The [Revision format](./revision-format.md) includes common
fields and ServiceStorage/resource declarations in one identity-bearing
projection. The [Revision Pack source](./source-format.md) is projected into this
format; it does not define another Revision identity or a raw-Core authoring
route. Changes to a published format follow the
[compatibility policy](../storage/compatibility.md).

**Verification: PR-TEST-0040, PR-TEST-0041, PR-TEST-0069, PR-TEST-0331, PR-TEST-0493, PR-TEST-0600.**

### PR-REQ-0015 - Runtime content closure identity

Runtime content identity MUST encode logical runtime location or role, object
kind, immutable content identity, and runtime-semantic metadata. Hashing an
unordered set of blob digests is insufficient. Source-only paths, timestamps,
inodes, and ownership metadata MUST NOT affect identity unless Pactrun
explicitly defines them as runtime semantics.

**Verification: PR-TEST-0044, PR-TEST-0045, PR-TEST-0069, PR-TEST-0600.**

### PR-REQ-0016 - Semantic normalization

Pactrun MUST apply semantic defaults, validate, and normalize equivalent forms
before projecting and encoding `RevisionCore`. Unordered domain collections
MUST be sorted by a stable semantic key; ordered collections MUST preserve their
order; duplicate semantic keys MUST be rejected.

Source acquisition MUST use the schema-directed scalar decoder in PR-REQ-0258 before this
normalization. A YAML library's implicit resolver or eager Boolean, null, or
numeric representation MUST NOT decide Revision meaning. Raw numeric source
tokens remain available until the target Frozen semantic type is known.

**Verification: PR-TEST-0041, PR-TEST-0045, PR-TEST-0068, PR-TEST-0333, PR-TEST-0337, PR-TEST-0494.**

### PR-REQ-0017 - Revision identity framing {#pr-req-0017---revisioncoreformatv1-hash-contract}

The Revision Core format MUST use a closed semantic schema, explicit domain
separation and framing, RFC 8785 JSON Canonicalization Scheme bytes, and a fixed
SHA-256 profile represented by a self-describing digest such as
`sha256:<value>`. JCS MUST NOT replace Pactrun semantic normalization.

The [Revision format](./revision-format.md) specifies the exact
encoding and digest framing.

**Verification: PR-TEST-0001, PR-TEST-0012, PR-TEST-0044, PR-TEST-0045.**

### PR-REQ-0018 - Stable published identity

Internal schema, type, serializer, library, and CLI changes MUST NOT alter the
identity of an already published RevisionCore format. Unknown or unsupported
format versions MUST be rejected instead of interpreted on a best-effort basis.

**Verification: PR-TEST-0044, PR-TEST-0331, PR-TEST-0338, PR-TEST-0339, PR-TEST-0493, PR-TEST-0494.**

### PR-REQ-0225 - Production opaque identity primitives

The production `PackageId`, `InstanceId`, and `InstanceStateVersion` types MUST
each contain 128 opaque bits and use exactly 32 lowercase hexadecimal
characters when rendered or parsed. Pactrun MUST generate new values using the
operating system cryptographic random source. `InstanceStateVersion` MUST NOT
expose arithmetic or ordering semantics.

`RevisionContentDigest` MUST retain its SHA-256 bytes. Its frozen canonical,
historical Hook and Snapshot codecs retain `sha256:` followed by 64 lowercase
hexadecimal characters. Current public Revision IDs use the same 64 digits
without that prefix under [the Revision Pack contract](./revision-packs.md).
`RevisionIdentity` remains an internal structured Package/version key. These
primitives do not define a generic Serde representation; each versioned codec
specifies its own representation. Changing text presentation MUST NOT rehash or
rewrite historical content.

**Verification: PR-TEST-0039.**

### PR-REQ-0226 - Production Revision content component boundary

The production Revision content model MUST preserve two sibling semantic and
hash components: `RevisionCoreV1` and
`RuntimeContentClosureIdentityV1`. `ValidatedRevisionContentV1` MUST contain the
validated pair. Neither component may contain the other, and neither is
persistence metadata. Cross-component Hook `ContentId` validation belongs to
the validated-pair boundary.

A Hook content reference exists when it resolves to exactly one entry
in the supplied semantic runtime-content closure. Direct launch additionally
requires that descriptor to be executable. Interpreter script launch does not
gain an executable-bit requirement. This validation MUST perform no filesystem,
content-store, materialization, permission, or launcher lookup.

Production projection MUST be deterministic and normalize the Frozen semantic
sets while preserving ordered Hook arguments. Semantic JSON decoding is an
internal codec and conformance facility, not a Revision Pack authoring, import, or
persistence contract. Revision Pack authoring MUST enter through
`NormalizedPackDefinition` and the typed projection boundary rather than this
internal parser.

**Verification: PR-TEST-0040, PR-TEST-0041, PR-TEST-0042.**

### PR-REQ-0227 - Production Revision Core codec conformance

The production wire view MUST contain exactly the identity-bearing fields in
Frozen `RevisionCoreFormatV1`. Typed invariants and implementation helpers MUST
NOT enter canonical JSON. In particular, runtime files serialize the Frozen
`id`, `path`, `kind: regular_file`, `blob_digest`, and `executable` fields;
runtime content has no independent format-version field.

Semantic decoding MUST reject duplicate decoded property names, unknown fields,
invalid UTF-8 or Unicode scalar sequences, invalid exact integers, non-finite
binary64 values, unsafe integers, and excessive input depth. Raw number tokens
MUST remain available until their schema type is known. Canonical production
decode MUST additionally require byte-for-byte equality with the re-encoded RFC
8785 JCS representation. The digest API MUST accept only a validated sibling
pair and apply the Frozen dual-component framing and SHA-256 profile.

Production code MUST NOT depend on or call Pactrun-owned xtask parser,
normalizer, or verifier implementation. Sharing suitable commodity Rust
dependencies does not violate this boundary. Every applicable Frozen negative
vector MUST be rejected, but an exact `PactrunErrorRef` is required only when
the vector's spelling is registered under `revision_core` in the
Frozen error catalog. Internal or verifier-only spellings MUST remain
outside the public error catalog and MUST NOT be mapped to a speculative generic error.

**Verification: PR-TEST-0040, PR-TEST-0043, PR-TEST-0044, PR-TEST-0045,
PR-TEST-0046.**
