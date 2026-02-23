# Work Plan: Accessing `round_committed` from FRIFoldProver

## **Understanding the Problem**

The `FRIFoldProver` struct in the binius crate maintains per-round commitment data in a field called `round_committed`. This data is currently **private** and not accessible outside the prove flow. You need to access this data for [insert your specific use case here - e.g., debugging, verification, analysis].

### **Current Architecture**

```
FRIFoldProver::new()
    ↓
execute_fold_round() [multiple times]
    - Each round: pushes (codeword, committed) to round_committed
    ↓
finalize()
    - Consumes FRIFoldProver
    - Returns FRIQueryProver containing round_committed
    ↓
prove_query() / other query methods
```

### **Data Flow of `round_committed`**

1. **Type**: `Vec<(FieldBuffer<F>, MerkleProver::Committed)>`
2. **Location**: Line 42 in `/Volumes/Personal/Avail/binius/binius64/crates/iop-prover/src/fri/fold.rs`
3. **Population**: Line 170 - `self.round_committed.push((folded_codeword, committed))`
4. **Transfer**: Line 206-215 - Moved to `FRIQueryProver` via destructuring
5. **Current Visibility**: `pub(super)` (module-private)

---

## **Recommended Solutions**

### **Option A: Add Public Getter to Binius Crate** ⭐ RECOMMENDED

**Approach**: Modify the binius source to expose the data.

**Implementation**:
1. Add a public method to `FRIQueryProver`:
```rust
// In binius/binius64/crates/iop-prover/src/fri/query.rs
impl<F, P, MerkleProver, VCS> FRIQueryProver<'_, F, P, MerkleProver, VCS> {
    /// Returns the round-by-round committed data
    pub fn round_committed(&self) -> &[(FieldBuffer<F>, MerkleProver::Committed)] {
        &self.round_committed
    }
}
```

2. Alternative: Add to `FRIFoldProver` before finalization:
```rust
// In binius/binius64/crates/iop-prover/src/fri/fold.rs
impl<...> FRIFoldProver<...> {
    /// Returns current round commitments (read-only)
    pub fn round_committed(&self) -> &[(FieldBuffer<F>, MerkleProver::Committed)] {
        &self.round_committed
    }
}
```

**Pros**:
- Cleanest API
- Direct access to the data
- No wrapper code needed in your project

**Cons**:
- Requires modifying external dependency
- May need to maintain a fork
- Changes may not be accepted upstream

**Steps**:
1. Fork/clone the binius repository
2. Add the getter method(s)
3. Update your Cargo.toml to use local path:
   ```toml
   binius-iop-prover = { path = "../binius/binius64/crates/iop-prover" }
   ```
4. Test and validate

---

### **Option B: Create Parallel Tracker in Your Codebase**

**Approach**: Mirror the folding rounds in your own code to capture the data.

**Implementation**:
```rust
// In your codebase (e.g., src/fri_tracker.rs)

pub struct RoundCommitmentTracker<F, Committed> {
    rounds: Vec<(FieldBuffer<F>, Committed)>,
}

impl<F, Committed> RoundCommitmentTracker<F, Committed> {
    pub fn new() -> Self {
        Self { rounds: Vec::new() }
    }
    
    pub fn track_round(&mut self, codeword: FieldBuffer<F>, committed: Committed) {
        self.rounds.push((codeword, committed));
    }
    
    pub fn get_rounds(&self) -> &[(FieldBuffer<F>, Committed)] {
        &self.rounds
    }
    
    pub fn get_round(&self, index: usize) -> Option<&(FieldBuffer<F>, Committed)> {
        self.rounds.get(index)
    }
}
```

**Integration**:
- You would manually call `track_round()` after each fold operation
- Requires modifying your prove flow to track alongside FRIFoldProver

**Pros**:
- No external changes needed
- Full control over the data structure
- Can add custom metadata

**Cons**:
- Duplicates logic from FRIFoldProver
- Must manually keep in sync
- More code to maintain

**Steps**:
1. Create the tracker struct
2. Modify your prove function to instantiate tracker
3. After each fold round, extract and track the data
4. Use tracker data after prove completes

---

### **Option C: Custom FRI Prover Wrapper**

**Approach**: Create a wrapper around FRIFoldProver that intercepts and stores the data.

**Implementation**:
```rust
pub struct InstrumentedFRIFoldProver<'a, F, P, NTT, MerkleProver> {
    inner: FRIFoldProver<'a, F, P, NTT, MerkleProver>,
    round_data: Vec<(FieldBuffer<F>, MerkleProver::Committed)>,
}

impl<...> InstrumentedFRIFoldProver<...> {
    pub fn new(inner: FRIFoldProver<...>) -> Self {
        Self {
            inner,
            round_data: Vec::new(),
        }
    }
    
    pub fn execute_fold_round(&mut self) -> Result<...> {
        // Call inner method
        let output = self.inner.execute_fold_round()?;
        
        // Extract and store the data (need to access via accessor or public field)
        // This requires modification to FRIFoldProver to expose round_committed
        
        Ok(output)
    }
    
    pub fn get_round_data(&self) -> &[...] {
        &self.round_data
    }
}
```

**Pros**:
- Keeps tracking logic encapsulated
- Can delegate to inner prover

**Cons**:
- Still requires some access to FRIFoldProver internals
- Complex wrapper structure

---

### **Option D: Patch Binius via Cargo Patch**

**Approach**: Use Cargo's patch feature to override the dependency.

**Implementation**:
1. Make local modifications to binius source (add getter method)
2. In your `Cargo.toml`:
```toml
[patch.crates-io]
binius-iop-prover = { path = "../binius/binius64/crates/iop-prover" }
```

**Pros**:
- Minimal changes to your project structure
- Can track upstream changes
- Clean upgrade path if accepted upstream

**Cons**:
- Still requires maintaining patch
- May break with upstream updates

---

## **Decision Matrix**

| Criterion | Option A | Option B | Option C | Option D |
|-----------|----------|----------|----------|----------|
| **Implementation Complexity** | Low | Medium | High | Low |
| **Maintenance Burden** | Medium | High | Medium | Medium |
| **Performance Impact** | None | Minimal | Minimal | None |
| **Flexibility** | High | High | Medium | High |
| **External Dependencies** | Fork binius | None | Modified binius | Patch binius |
| **Code Clarity** | High | Medium | Low | High |

---

## **Recommended Action Plan**

### **Phase 1: Validate the Approach** (15 min)
1. Confirm which data from `round_committed` you need:
   - Just the codewords?
   - Just the Merkle commitments?
   - Both?
   - How will you use the data?

2. Check if binius crate is already a local dependency:
   - Look at `Cargo.toml` in `/Volumes/Personal/Avail/binius-das-poc/`
   - Check if binius is using `path = ` or `git = ` dependencies

### **Phase 2: Choose and Implement** (1-2 hours)

**If binius is already local/modifiable:** → **Option A**
- Add getter method to `FRIQueryProver`
- Run tests
- Create PR or maintain local branch

**If binius is external and you don't want to modify:** → **Option B**
- Create tracker struct
- Modify your prove flow
- Test integration

### **Phase 3: Testing** (30 min)
1. Verify you can access the data after prove
2. Check data integrity (rounds match expected count)
3. Ensure no performance regression

---

## **Next Steps**

**Immediate Questions**:
1. What specific data do you need from `round_committed`?
2. Is binius currently a local dependency or from crates.io?
3. Are you comfortable maintaining a fork/patch, or do you prefer no external changes?

**Ready to proceed?** Tell me:
- Which option you prefer
- What the data will be used for
- If you'd like me to draft the implementation code

Then run `/start-work` and I'll execute the chosen approach.