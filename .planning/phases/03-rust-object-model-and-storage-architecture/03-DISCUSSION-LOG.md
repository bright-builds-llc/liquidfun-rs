# Phase 3: Rust Object Model and Storage Architecture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-10
**Phase:** 3-rust-object-model-and-storage-architecture
**Mode:** Yolo
**Areas discussed:** Typed handle identity and invalidation; callback, destruction, and mutation boundary; stable storage, user data, and external buffers

***

## Typed Handle Identity and Invalidation

| Option | Description | Selected |
| --- | --- | --- |
| Custom world-tagged generational arena | Distinct opaque handles carry world, private slot, and wide generation; the engine owns exhaustion and iteration semantics. | ✓ |
| Never-reused monotonic typed IDs | Identity is independent of storage through an ID-to-location map with no generation wrap. | |
| `slotmap` typed keys plus world tag | Reuse a mature arena dependency but accept or separately mitigate its documented version-wrap and map-boundary semantics. | |

**Agent's choice:** Custom world-tagged generational arena.
**Notes:** Selected for a literal never-resurrect API-02 guarantee, explicit cross-world errors, private ordering control, and no new runtime dependency. Use `u64` generations and retire exhausted slots.

***

## Callback, Destruction, and Mutation Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Borrow-scoped synchronous hooks, owned `StepReport`, deferred commands | Preserve upstream decision timing while preventing mutable-world access and escaped contact references. | ✓ |
| Synchronous directives plus post-step events only | Simplify reentrancy and panic handling but intentionally delay observational callback timing. | |
| C++-style mutable listeners | Closely mirror upstream but expose live transient state and require interior mutability/reentrancy machinery. | |

**Agent's choice:** Borrow-scoped synchronous hooks, owned `StepReport`, and deferred commands.
**Notes:** Preserve event order and multiplicity, apply commands after unlock with fresh handle validation, and poison the world after a hook panic during a partial step.

***

## Stable Storage, User Data, and External Buffers

| Option | Description | Selected |
| --- | --- | --- |
| Dense SoA plus custom generational identity registry | Stable public IDs remain separate from dense solver indices; one permutation updates all lanes and derived references. | ✓ |
| Dense SoA plus monotonic never-reused IDs | Avoid generation wrap at the cost of permanently growing identity metadata and another lookup policy. | |
| Stable arena plus dense solver projection | Make stable slots canonical and synchronize a second dense representation for solver work. | |

**Agent's choice:** Dense SoA plus custom generational identity registry.
**Notes:** Add explicit live/pending-delete/vacant states, typed application-owned association side tables, and an owned validated fixed-capacity buffer-bundle contract for future Phase 9 completion.

## Agent's Discretion

- Exact private naming, opaque diagnostic formatting, representative lanes, test-operation weighting, and cleanup-helper ergonomics.

## Deferred Ideas

- Full particle bulk mutation and external-buffer APIs remain Phase 9 work.
- Public handle serialization remains post-parity unless later requirements establish it.
- Unsafe raw-pointer interop requires a separate measured need and safety review.
