# Phase 35: Toast Limit & Node Polling - FAILED ❌

**Date**: 2025-11-10
**Goal**: Fix toast duplication (Tests 4, 6) and ReactFlow node detection (Test 2) by:
1. Limiting concurrent toasts to 1 (`TOAST_LIMIT = 1`)
2. Using direct DOM node polling instead of attribute-based detection

**Result**: **4/8 passing (50%)** - REGRESSION from Phase 34 (5/8 = 62.5%)

---

## 📊 Test Results Comparison

| Phase | Passing Tests | Pass Rate | Change |
|-------|--------------|-----------|--------|
| **Phase 34** | 5/8 | 62.5% | Baseline |
| **Phase 35** | 4/8 | 50.0% | ❌ -12.5% (worse!) |

### Detailed Test Status

| Test | Phase 34 | Phase 35 | Change | Issue |
|------|----------|----------|--------|-------|
| 1. Pattern 모드 - 단순 워크플로우 | ✅ | ✅ | - | - |
| 2. LLM 모드 - 복잡한 워크플로우 | ❌ | ❌ | - | Node detection |
| **3. Hybrid 모드 - Pattern 성공** | **✅** | **❌** | **REGRESSION** | **New failure!** |
| 4. Hybrid 모드 - LLM 보완 | ❌ | ❌ | - | Toast duplication |
| 5. Pattern 모드 - 단순 생성 | ✅ | ✅ | - | - |
| 6. 잘못된 API 키 에러 처리 | ❌ | ❌ | - | Toast duplication |
| 7. Pattern 모드 - 빈 입력 처리 | ✅ | ✅ | - | - |
| 8. Pattern 매칭 불가 시나리오 | ✅ | ✅ | - | - |

---

## 🔧 Changes Made

### 1. Toast Limit Reduction
**File**: `src/components/ui/use-toast.ts`
**Change**: Line 5
```typescript
// BEFORE:
const TOAST_LIMIT = 3

// AFTER:
const TOAST_LIMIT = 1
```

**Hypothesis**: Limiting to 1 concurrent toast would prevent duplication.
**Result**: **FAILED** ❌ - Toast duplication persists.

### 2. ReactFlow Node Polling
**File**: `tests/e2e/workflow-generation.spec.ts`
**Changes**: Tests 1, 2, 4, 5 - replaced attribute-based detection with direct DOM polling

**Example (Test 2, lines 177-190)**:
```typescript
// BEFORE (Phase 34):
await page.waitForFunction(() => {
  return document.body.getAttribute('data-reactflow-ready') === 'true';
}, { timeout: 10000 });

// AFTER (Phase 35):
console.log('[Test 2] Waiting for ReactFlow nodes to render (expected: >= 4)...');
await page.waitForFunction((expectedCount) => {
  const nodes = document.querySelectorAll('.react-flow__node');
  return nodes.length >= expectedCount;
}, 4, { timeout: 10000 });
console.log('[Test 2] ReactFlow nodes detected!');
```

**Hypothesis**: Direct node counting would be more reliable than attribute detection.
**Result**: **PARTIALLY FAILED** ❌ - Test 2 still fails, and Test 3 regressed!

### 3. Code Cleanup
**File**: `src/pages/WorkflowBuilder.tsx`
**Removed**:
- `isReactFlowReady` state (line 73-74)
- Node change tracking useEffect (lines 132-146)
- Enhanced ReactFlow init handler (lines 604-616)
- Reverted `onInit={handleReactFlowInit}` to `onInit={setReactFlowInstance}` (line 1238)

**Purpose**: Clean up Phase 34's failed rendering detection approach.
**Result**: ✅ Successfully removed unused code.

---

## ❌ Why Phase 35 Failed

### Problem 1: Toast Duplication Root Cause Misidentified

**What We Thought**: Too many toasts queued → reduce TOAST_LIMIT to 1
**Reality**: shadcn/ui toast renders **TWO DOM elements per toast**:
1. Visual content: `<div class="text-sm font-semibold">✨ 워크플로우 생성 완료</div>`
2. ARIA live region: `<span role="status" aria-live="assertive">Notification...</span>`

**Test 4 Error**:
```
Error: strict mode violation: locator('text=워크플로우 생성 완료') resolved to 2 elements:
    1) <div class="text-sm font-semibold">✨ 워크플로우 생성 완료</div>
    2) <span role="status" aria-live="assertive">Notification 워크플로우 생성 완료...</span>
```

**Test 6 Error**:
```
Error: strict mode violation: locator('text=API 키가 유효하지 않습니다').or(locator('text=Invalid Claude API key')) resolved to 2 elements:
    1) <div class="text-sm opacity-90">API 키가 유효하지 않습니다. 확인 후 다시 시도해주세요.</div>
    2) <span role="status" aria-live="assertive">Notification 생성 실패API 키가 유효하지 않습니다...
```

**Conclusion**: TOAST_LIMIT doesn't address dual-element rendering. Need to modify **test selectors**, not toast configuration.

---

### Problem 2: Node Polling Strategy Caused Regression

**Test 3 (Hybrid Pattern) - NEW FAILURE**:
- Phase 34: ✅ PASSING (with attribute-based detection)
- Phase 35: ❌ FAILING (with node polling)

**Hypothesis on Why Test 3 Regressed**:
1. The `waitForFunction` might be too aggressive (immediate node check without delay)
2. Pattern mode might generate nodes differently than LLM/Hybrid mode
3. Phase 34's 100ms delay (`setTimeout`) might have been beneficial for Test 3

**Test 2 (LLM Mode) - Still Failing**:
- Node polling didn't help detect nodes correctly
- Possible timing issue with complex workflow generation

---

## 🔍 Error Analysis

### Toast Duplication (Tests 4, 6)
**Root Cause**: shadcn/ui Toaster component creates dual elements for accessibility:
- Visual toast (div)
- Screen reader announcement (span with role="status")

**Why It Matters**: Playwright's strict mode requires locators to resolve to **exactly 1 element**. Text-based locators match BOTH elements, causing "strict mode violation".

**Fix Strategy**:
1. Use more specific selectors (target div, not text)
2. Or use `.first()` to select the first match
3. Or modify toast rendering to avoid dual elements

---

### ReactFlow Node Detection (Tests 2, 3)
**Test 2 Issue**: Still not detecting 4+ nodes after generation
**Test 3 Issue**: Regressed from Phase 34 (was passing!)

**Possible Causes**:
1. **Timing**: Direct node polling might execute before ReactFlow renders
2. **Mock Response**: LLM mode mock might not be triggering node generation
3. **Pattern Mode Difference**: Test 3's Pattern mode might need different handling

**Evidence from Test 3 Error Context**:
- Page snapshot shows nodes exist: "워크플로우 캔버스" section visible
- But test still fails - suggests timing issue or selector problem

---

## 📝 Lessons Learned

### 1. Understanding Toast Architecture is Critical
- Don't assume toast duplication = too many toasts queued
- shadcn/ui creates 2 elements by design (visual + ARIA)
- Need to understand component internals before fixing

### 2. "Simpler" Strategies Can Backfire
- Direct DOM polling seemed simpler than attribute-based detection
- But caused regression in Test 3 (which was passing in Phase 34)
- Sometimes the "hacky" solution (100ms delay) works better

### 3. Test Stability > Purity
- Phase 34's approach (attribute + delay) was working for Test 3
- Phase 35's "cleaner" approach (direct polling) broke it
- **Conclusion**: Don't fix what ain't broke!

---

## 🎯 Phase 36 Strategy (Recommendation)

### Approach 1: Fix Toast Selectors (High Priority)
**Target**: Tests 4, 6
**Changes**: Modify test selectors to target specific toast elements

**Example**:
```typescript
// INSTEAD OF:
await expect(page.locator('text=워크플로우 생성 완료')).toBeVisible();

// USE:
await expect(page.locator('.toast div.text-sm').filter({ hasText: '워크플로우 생성 완료' })).toBeVisible();
```

**Expected Result**: Tests 4, 6 should pass (no more strict mode violations)

---

### Approach 2: Revert Node Polling for Test 3 (Medium Priority)
**Target**: Test 3
**Changes**: Restore Phase 34's attribute-based detection for Pattern mode tests

**Rationale**: Test 3 was passing in Phase 34, so reverting might restore it.

**Expected Result**: Test 3 should pass again (6/8 total)

---

### Approach 3: Debug Test 2 LLM Mode (Low Priority)
**Target**: Test 2
**Investigation Needed**:
1. Verify mock response structure generates nodes correctly
2. Add longer wait time after "AI로 생성" button click
3. Check if LLM mode mock is actually being applied

**Expected Result**: Test 2 should pass (7/8 total)

---

## 📊 Expected Phase 36 Outcome

If all approaches succeed:
- **Fix toast selectors**: Tests 4, 6 pass (+2)
- **Revert Test 3 detection**: Test 3 passes (+1)
- **Debug Test 2**: Test 2 passes (+1)

**Target**: **8/8 passing (100%)** 🎯

---

## 🔧 Git Changes

### Modified Files
1. `src/components/ui/use-toast.ts` - TOAST_LIMIT = 1 (INEFFECTIVE)
2. `tests/e2e/workflow-generation.spec.ts` - Node polling strategy (CAUSED REGRESSION)
3. `src/pages/WorkflowBuilder.tsx` - Cleanup (NO IMPACT)

### Test Results
- `phase35-test.log` - 4/8 passing (50%)
- Test failure screenshots captured in `test-results/`

---

## 🚨 Critical Insight

**Phase 35 made things WORSE (-12.5% pass rate).**

**Key Takeaway**:
1. Toast duplication is NOT about queue management - it's about dual-element rendering
2. Direct node polling is NOT always better than attribute-based detection
3. Need to understand WHY Phase 34's approach worked for Test 3 before replacing it

**Next Step**: Implement Phase 36 with targeted selector fixes instead of broad architectural changes.
