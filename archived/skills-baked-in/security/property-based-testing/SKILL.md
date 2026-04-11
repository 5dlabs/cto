---
name: property-based-testing
description: Property-based testing for stronger coverage than example tests - roundtrip, idempotence, invariants.
---

# Property-Based Testing Guide

Use this skill when you encounter patterns where PBT provides stronger coverage than example-based tests.

## When to Invoke (Automatic Detection)

**Invoke this skill when you detect:**
- **Serialization pairs**: `encode`/`decode`, `serialize`/`deserialize`, `toJSON`/`fromJSON`
- **Parsers**: URL parsing, config parsing, protocol parsing
- **Normalization**: `normalize`, `sanitize`, `clean`, `canonicalize`
- **Validators**: `is_valid`, `validate`, `check_*`
- **Data structures**: Custom collections with `add`/`remove`/`get` operations
- **Mathematical/algorithmic**: Pure functions, sorting, ordering, comparators

## When NOT to Use

- Simple CRUD operations without transformation logic
- One-off scripts or throwaway code
- Code with side effects that cannot be isolated
- Tests where specific example cases are sufficient
- Integration or end-to-end testing

## Property Catalog (Quick Reference)

| Property | Formula | When to Use |
|----------|---------|-------------|
| **Roundtrip** | `decode(encode(x)) == x` | Serialization, conversion pairs |
| **Idempotence** | `f(f(x)) == f(x)` | Normalization, formatting, sorting |
| **Invariant** | Property holds before/after | Any transformation |
| **Commutativity** | `f(a, b) == f(b, a)` | Binary/set operations |
| **Associativity** | `f(f(a,b), c) == f(a, f(b,c))` | Combining operations |
| **Identity** | `f(x, identity) == x` | Operations with neutral element |
| **Inverse** | `f(g(x)) == x` | encrypt/decrypt, compress/decompress |
| **Oracle** | `new_impl(x) == reference(x)` | Optimization, refactoring |
| **Easy to Verify** | `is_sorted(sort(x))` | Complex algorithms |
| **No Exception** | No crash on valid input | Baseline property |

**Strength hierarchy** (weakest to strongest):
No Exception → Type Preservation → Invariant → Idempotence → Roundtrip

## Priority by Pattern

| Pattern | Property | Priority |
|---------|----------|----------|
| encode/decode pair | Roundtrip | HIGH |
| Pure function | Multiple | HIGH |
| Validator | Valid after normalize | MEDIUM |
| Sorting/ordering | Idempotence + ordering | MEDIUM |
| Normalization | Idempotence | MEDIUM |
| Builder/factory | Output invariants | LOW |

## Examples by Language

### Python (Hypothesis)

```python
from hypothesis import given, strategies as st

@given(st.binary())
def test_roundtrip(data):
    assert decode(encode(data)) == data

@given(st.text())
def test_normalize_idempotent(s):
    assert normalize(normalize(s)) == normalize(s)

@given(st.lists(st.integers()))
def test_sort_idempotent(xs):
    assert sorted(sorted(xs)) == sorted(xs)
```

### TypeScript (fast-check)

```typescript
import fc from 'fast-check';

test('roundtrip', () => {
  fc.assert(fc.property(fc.uint8Array(), (data) => {
    expect(decode(encode(data))).toEqual(data);
  }));
});

test('normalize is idempotent', () => {
  fc.assert(fc.property(fc.string(), (s) => {
    expect(normalize(normalize(s))).toBe(normalize(s));
  }));
});
```

### Rust (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip(data: Vec<u8>) {
        prop_assert_eq!(decode(&encode(&data)), data);
    }
    
    #[test]
    fn normalize_idempotent(s: String) {
        let once = normalize(&s);
        let twice = normalize(&once);
        prop_assert_eq!(once, twice);
    }
}
```

### Go (rapid)

```go
func TestRoundtrip(t *testing.T) {
    rapid.Check(t, func(t *rapid.T) {
        data := rapid.SliceOf(rapid.Byte()).Draw(t, "data")
        decoded := Decode(Encode(data))
        if !bytes.Equal(decoded, data) {
            t.Fatalf("roundtrip failed")
        }
    })
}
```

## How to Suggest PBT

When you detect a high-value pattern while writing tests, **offer PBT as an option**:

> "I notice `encode_message`/`decode_message` is a serialization pair. Property-based testing with a roundtrip property would provide stronger coverage than example tests. Want me to use that approach?"

**If codebase already uses a PBT library** (Hypothesis, fast-check, proptest), be more direct:

> "This codebase uses Hypothesis. I'll write property-based tests for this serialization pair using a roundtrip property."

**If user declines**, write good example-based tests without further prompting.

## Red Flags

- Recommending trivial getters/setters
- Missing paired operations (encode without decode)
- Ignoring type hints (well-typed = easier to test)
- Overwhelming user with candidates (limit to top 5-10)
- Being pushy after user declines

## PBT Libraries by Language

| Language | Library | Install |
|----------|---------|---------|
| Python | Hypothesis | `pip install hypothesis` |
| TypeScript/JS | fast-check | `npm install fast-check` |
| Rust | proptest | `cargo add proptest --dev` |
| Go | rapid | `go get pgregory.net/rapid` |
| Haskell | QuickCheck | `cabal install QuickCheck` |
| Scala | ScalaCheck | sbt dependency |
| Elixir | StreamData | mix dependency |

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) property-based-testing skill.
