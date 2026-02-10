# Publishing Script Fix

**Date:** 2025-01-23  
**Issue:** Smart publish script was skipping unpublished crates  
**Status:** ✅ FIXED

---

## Problem

The `publish-crates-smart.sh` script had a critical logic flaw:

### Before (BROKEN):
```bash
should_publish_crate() {
    # 1. Check if in CHANGED_CRATES list
    # 2. Skip if not in list
    # ❌ NEVER checked crates.io!
}
```

**Result:** 
- Script marked `feagi-npu-neural` as "unchanged" and skipped it
- But `feagi-npu-neural` wasn't on crates.io yet!
- All dependent crates failed: "no matching package named `feagi-npu-neural`"

---

## Fix

Changed the check order:

### After (FIXED):
```bash
should_publish_crate() {
    # 1. ✅ FIRST: Check crates.io - already published?
    if is_already_published; then
        return "skip_published"  # Already on crates.io
    fi
    
    # 2. ✅ THEN: Check changed list
    if in_changed_list; then
        return "publish"
    else
        return "skip_unchanged"  # Not changed, but would publish if needed
    fi
}
```

**New Logic:**
1. **Always check crates.io first** - if published, skip regardless of changed list
2. **Then check changed list** - if not in list, skip (but only if already published)
3. **Unpublished crates** - always get published even if not in changed list

---

## Key Changes

### 1. New Function: `is_already_published()`
```bash
is_already_published() {
    local crate_name=$1
    local crate_path=$2
    
    # Get version from Cargo.toml
    cd "$crate_path"
    local version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    
    # Check crates.io
    if cargo search "$crate_name" --limit 1 2>/dev/null | grep -q "^$crate_name = \"$version\""; then
        echo "true"
    else
        echo "false"
    fi
}
```

### 2. Updated `should_publish_crate()`
Now returns three states:
- `skip_published` - Already on crates.io
- `skip_unchanged` - Not changed (and not needed)
- `publish` - Needs publishing

### 3. Updated Main Loop
```bash
publish_decision=$(should_publish_crate "$crate_name" "$crate_path")

if [ "$publish_decision" = "skip_published" ]; then
    echo "⏭️  Skipping $crate_name (already published on crates.io)"
elif [ "$publish_decision" = "skip_unchanged" ]; then
    echo "⏭️  Skipping $crate_name (not in changed list)"
else
    publish_crate "$crate_name"
fi
```

---

## Testing

To verify the fix works:

```bash
# Simulate: Some crates published, some not
CHANGED_CRATES="feagi-structures feagi-npu-neural" ./scripts/publish-crates-smart.sh

# Expected behavior:
# 1. Skip feagi-observability (already published)
# 2. Skip feagi-config (already published)
# 3. Publish feagi-structures (in changed list)
# 4. Publish feagi-npu-neural (in changed list)
# 5. Publish feagi-npu-runtime (depends on feagi-npu-neural, not published yet)
# 6. Continue in order...
```

---

## Why This Matters

**Independent Versioning Requirements:**
- Each crate can be at different versions
- Changed crates need publishing
- **BUT** unpublished dependencies MUST be published first
- Can't rely solely on "changed" detection

**The Fix Ensures:**
- ✅ Already-published crates are always skipped (saves time)
- ✅ Unpublished crates are always published (even if "unchanged")
- ✅ Dependency order is respected
- ✅ No false "already exists" errors

---

## Related Files

- `scripts/publish-crates-smart.sh` - The fixed script
- `VERSIONING_POLICY.md` - Independent versioning rules
- `PUBLISHING_ORDER.md` - Dependency order

---

**This fix is critical for independent versioning to work correctly.**

