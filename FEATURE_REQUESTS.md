# Feature Requests

## Arrow Key Path Navigation (Fish/zsh-style)

**Status:** Planned
**Priority:** Medium
**Requested:** 2024-12-31

### Current Behavior

When typing a path and using tab completion:
```
Type: ~/Lib[Tab]
Result: ~/Library/
```

- Tab: Accept completion and add `/`
- Type more to continue

### Requested Behavior

**Fish shell-style inline completion with arrow navigation:**

```
Type: ~/Lib
Shows: ~/Library          ← Ghost text preview

Press →: Accept "~/Library/" and continue typing
Press ←: Reject completion, backspace
Press Tab: Same as → (accept and continue)
```

### Use Case

**Current workflow (3 steps):**
1. Type `~/Lib`
2. Press Tab → `~/Library/`
3. Type `Mob[Tab]` → `~/Library/Mobile Documents/`

**Desired workflow (smoother):**
1. Type `~/Lib` → see ghost text `~/Library`
2. Press → to accept → `~/Library/`
3. Type `Mob` → see ghost text `~/Library/Mobile Documents`
4. Press → to accept
5. Walk through path segments with arrow keys

### Implementation Notes

**Arrow key behavior:**
- **→ (Right)**: Accept current fuzzy match / inline completion
- **← (Left)**: Navigate backwards in path segments? Or just backspace?
- **↑/↓**: Navigate match list (already implemented ✅)

**Similar to:**
- Fish shell's inline autosuggestions
- zsh with zsh-autosuggestions plugin
- VS Code path completion

**Technical approach:**
1. Show first fuzzy match as ghost text (dim/grey)
2. Right arrow accepts ghost text and adds `/`
3. Update fuzzy matches as you type
4. Keep current ↑↓ list navigation

**Benefits:**
- Faster path entry (fewer keypresses)
- Visual feedback of available completions
- More intuitive for users familiar with Fish/zsh
- Better for long nested paths (common in macOS)

### Related Issues

- Current max_depth(3) should handle nested projects like:
  ```
  byte_tooling/
    byte/
      byte.toml   ← depth 2, found ✅
  ```

### Files to Modify

- `src/tui/mod.rs` - Add ghost text rendering
- `src/tui/mod.rs` - Update arrow key handlers
- Consider adding inline preview to fuzzy matches display
