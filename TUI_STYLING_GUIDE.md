# Byte TUI Styling Guide

This document defines the visual design language for Byte's Terminal User Interface. All UI work should reference this guide to maintain consistency and quality.

## Design Philosophy

Byte's TUI embodies these principles:

1. **Clarity First**: Information hierarchy must be immediately obvious
2. **OLED-Optimized**: Colors chosen for maximum readability on pure black backgrounds
3. **Breathing Room**: Generous whitespace and padding prevent visual clutter
4. **Subtle by Default**: Visual elements should guide, not distract
5. **Modern CLI Aesthetic**: Inspired by tools like GitHub CLI, Vercel CLI, and Stripe CLI

---

## Color Palette

All colors are defined in `src/tui/mod.rs` in the `theme` module. **Always use theme constants, never hardcode colors.**

### Primary Colors

```rust
theme::ACCENT      // Cyan - RGB(0, 255, 255)
theme::SUCCESS     // Green - RGB(0, 255, 0)
theme::ERROR       // Red - RGB(255, 0, 0)
```

**Usage:**
- `ACCENT`: Brand color, active states, primary interactive elements, selected items
- `SUCCESS`: Confirmation messages, active drivers, positive states
- `ERROR`: Error messages, warnings, destructive actions

### Text Hierarchy

Optimized for OLED black backgrounds with proper contrast ratios:

```rust
theme::TEXT_PRIMARY     // White - RGB(255, 255, 255)
theme::TEXT_SECONDARY   // Light Gray - RGB(160, 160, 160)
theme::TEXT_TERTIARY    // Medium Gray - RGB(140, 140, 140)
theme::TEXT_DIM         // Subtle Gray - RGB(120, 120, 120)
```

**Usage Guidelines:**

| Level | Color | Use For | Example |
|-------|-------|---------|---------|
| Primary | `TEXT_PRIMARY` | Page titles, main content, unselected items | Project names, command names |
| Secondary | `TEXT_SECONDARY` | Descriptions, body text, subtitles | Project descriptions, status messages |
| Tertiary | `TEXT_TERTIARY` | Labels, metadata, counts, tags | "PATH", "DRIVERS", driver hashtags |
| Dim | `TEXT_DIM` | Help text, hints, very subtle information | Keyboard shortcuts in footer |

**Contrast Requirements:**
- Primary text: WCAG AAA (>7:1 contrast ratio)
- Secondary text: WCAG AA (>4.5:1 contrast ratio)
- Tertiary text: WCAG AA for large text (>3:1 contrast ratio)

### UI Elements

```rust
theme::SEPARATOR      // RGB(60, 60, 60) - Lines, dividers
theme::HIGHLIGHT_BG   // RGB(40, 40, 40) - Selected backgrounds (not currently used)
theme::BADGE_BG       // Cyan - Badge backgrounds
theme::BADGE_TEXT     // Black - Badge text
```

---

## Typography

### Text Styles

```rust
// Primary heading
Style::default()
    .fg(theme::TEXT_PRIMARY)
    .add_modifier(Modifier::BOLD)

// Secondary text
Style::default()
    .fg(theme::TEXT_SECONDARY)

// Subtle/dim text
Style::default()
    .fg(theme::TEXT_TERTIARY)
    .add_modifier(Modifier::DIM)

// Active/selected item
Style::default()
    .fg(theme::ACCENT)
    .add_modifier(Modifier::BOLD)

// Badge/pill
Style::default()
    .fg(theme::BADGE_TEXT)
    .bg(theme::BADGE_BG)
    .add_modifier(Modifier::BOLD)
```

### Hierarchy Patterns

**Page Title Pattern:**
```rust
Line::from(vec![
    Span::styled("Section Name", Style::default()
        .fg(theme::TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)),
    Span::raw("  "),
    Span::styled("3", Style::default()
        .fg(theme::TEXT_TERTIARY)),
])
```

**List Item Pattern (Two-line):**
```rust
// Line 1: Title (selectable)
Line::from(vec![
    Span::raw("  "),
    Span::styled(title, Style::default()
        .fg(if selected { theme::ACCENT } else { theme::TEXT_PRIMARY })
        .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() })),
])

// Line 2: Description + metadata
Line::from(vec![
    Span::raw("  "),
    Span::styled(description, Style::default()
        .fg(theme::TEXT_SECONDARY)),
    Span::raw("  "),
    Span::styled(metadata, Style::default()
        .fg(theme::TEXT_TERTIARY)
        .add_modifier(Modifier::DIM)),
])
```

**Section Header Pattern:**
```rust
Line::from(vec![
    Span::styled("SECTION LABEL", Style::default()
        .fg(theme::TEXT_TERTIARY)
        .add_modifier(Modifier::DIM)),
])
```

---

## Layout & Spacing

### Margins

Use `inner()` with `Margin` to create breathing room:

```rust
let inner_area = area.inner(Margin {
    horizontal: 2,  // 2 characters on each side
    vertical: 1,    // 1 line top and bottom
});
```

**Standard margins:**
- Content areas: `horizontal: 2, vertical: 1`
- Dense areas: `horizontal: 1, vertical: 0`
- Spacious areas: `horizontal: 3, vertical: 2`

### Vertical Spacing

Add empty lines between logical sections:

```rust
Line::from(""),      // Single line spacing
Line::from(""),      // Double line spacing for major sections
```

**Spacing rules:**
- Between list items: 1 empty line
- Between sections: 2 empty lines
- Before section headers: 2 empty lines
- After section headers: 1 empty line

### Horizontal Spacing

```rust
Span::raw("  ")      // 2 spaces: standard indent
Span::raw("    ")    // 4 spaces: nested indent
```

---

## Components

### Header

Centered branding with separator:

```rust
Line::from(vec![
    Span::raw("  "),
    Span::styled("●", Style::default().fg(theme::ACCENT)),
    Span::raw("  "),
    Span::styled("B Y T E", Style::default()
        .fg(theme::ACCENT)
        .add_modifier(Modifier::BOLD)),
    Span::raw("  "),
    Span::styled("│", Style::default().fg(theme::SEPARATOR)),
    Span::raw("  "),
    Span::styled("Project Orchestration", Style::default()
        .fg(theme::TEXT_SECONDARY)),
])
```

### Tab Bar

Active tab gets cyan badge, inactive tabs are dimmed:

```rust
// Active tab
Span::styled("1", Style::default()
    .fg(theme::BADGE_TEXT)
    .bg(theme::BADGE_BG)
    .add_modifier(Modifier::BOLD))
Span::raw(" ")
Span::styled("Projects", Style::default()
    .fg(theme::ACCENT)
    .add_modifier(Modifier::BOLD))

// Inactive tab
Span::styled("2", Style::default()
    .fg(theme::TEXT_TERTIARY))
Span::raw(" ")
Span::styled("Commands", Style::default()
    .fg(theme::TEXT_TERTIARY))
```

### Footer

Left-aligned status with subtle help hints:

```rust
Line::from(vec![
    Span::raw("  "),
    Span::styled(status_message, status_style),
    Span::raw("  "),
    Span::styled("│", Style::default().fg(theme::SEPARATOR)),
    Span::raw("  "),
    Span::styled("?", Style::default()
        .fg(theme::TEXT_DIM)
        .add_modifier(Modifier::DIM)),
    Span::styled(" help", Style::default()
        .fg(theme::TEXT_DIM)
        .add_modifier(Modifier::DIM)),
    Span::raw("  "),
    Span::styled("q", Style::default()
        .fg(theme::TEXT_DIM)
        .add_modifier(Modifier::DIM)),
    Span::styled(" quit", Style::default()
        .fg(theme::TEXT_DIM)
        .add_modifier(Modifier::DIM)),
])
```

### Separators

Use subtle horizontal rules:

```rust
Line::from(vec![
    Span::styled("─".repeat(width as usize),
        Style::default().fg(theme::SEPARATOR).add_modifier(Modifier::DIM)),
])
```

### Lists

**Always use borderless blocks:**
```rust
List::new(items)
    .block(Block::default().borders(Borders::NONE))
    .highlight_style(Style::default())
    .highlight_symbol("▸ ")
```

**Selection indicator:** Use `▸` (not `→` or `•`)

### Status Indicators

```rust
// Active/success
Span::styled("●", Style::default().fg(theme::SUCCESS))

// Inactive/neutral
Span::styled("○", Style::default().fg(theme::TEXT_TERTIARY))

// Error/warning
Span::styled("●", Style::default().fg(theme::ERROR))
```

---

## Anti-Patterns

### Don't Do This ❌

```rust
// ❌ Hardcoded colors
.fg(Color::Gray)
.fg(Color::DarkGray)

// ❌ Heavy borders everywhere
Block::default().borders(Borders::ALL)

// ❌ No spacing between elements
Line::from(vec![
    Span::styled("Label:", ...),
    Span::styled("Value", ...),  // Cramped!
])

// ❌ Inconsistent selection styles
.bg(Color::DarkGray)  // Wrong background color

// ❌ Mixed text hierarchy
Span::styled("Important", Style::default().fg(Color::White))
Span::styled("Also important", Style::default().fg(Color::Cyan))
```

### Do This Instead ✅

```rust
// ✅ Use theme constants
.fg(theme::TEXT_SECONDARY)
.fg(theme::TEXT_TERTIARY)

// ✅ Borderless with margins
Block::default().borders(Borders::NONE)
area.inner(Margin { horizontal: 2, vertical: 1 })

// ✅ Proper spacing
Line::from(vec![
    Span::styled("Label:", ...),
    Span::raw("  "),  // Breathing room
    Span::styled("Value", ...),
])

// ✅ Consistent selection
.fg(if selected { theme::ACCENT } else { theme::TEXT_PRIMARY })
.add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() })

// ✅ Proper hierarchy
Span::styled("Primary", Style::default().fg(theme::TEXT_PRIMARY))
Span::styled("Secondary", Style::default().fg(theme::TEXT_SECONDARY))
```

---

## Testing on OLED

When implementing UI changes, always test on:

1. **OLED display** (if available) - pure black background
2. **Terminal with black background** - most common
3. **Terminal with dark background** - slightly gray
4. **Terminal with custom theme** - ensure colors override properly

**Color contrast checker:** Verify all text meets WCAG standards at https://webaim.org/resources/contrastchecker/

---

## Code Organization

### Theme Location

All theme colors are defined in:
```
src/tui/mod.rs
└── mod theme { ... }
```

### When to Update Theme

Update the theme module when:
- Adding new semantic colors (e.g., `WARNING`, `INFO`)
- Adjusting brightness for better contrast
- Adding UI-specific colors (e.g., `BORDER`, `SHADOW`)

**Never** add colors for specific components - use semantic naming.

### Adding New Colors

1. Add constant to theme module with RGB values
2. Add descriptive comment
3. Document in this guide under appropriate section
4. Update any affected components
5. Test on OLED display

Example:
```rust
pub const WARNING: Color = Color::Rgb(255, 165, 0);  // Warning states, caution messages
```

---

## Examples

### Complete View Example

```rust
fn render_example_view(f: &mut Frame, area: Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Title section
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Section Title", Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("5", Style::default()
                .fg(theme::TEXT_TERTIARY)),
        ]),
        Line::from(""),
    ]);

    // Content section
    let content_lines = vec![
        Line::from(vec![
            Span::styled("METADATA", Style::default()
                .fg(theme::TEXT_TERTIARY)
                .add_modifier(Modifier::DIM)),
        ]),
        Line::from(vec![
            Span::styled("Value goes here", Style::default()
                .fg(theme::TEXT_PRIMARY)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ●", Style::default().fg(theme::SUCCESS)),
            Span::raw("  "),
            Span::styled("Active item", Style::default()
                .fg(theme::TEXT_PRIMARY)),
            Span::raw("  "),
            Span::styled("metadata", Style::default()
                .fg(theme::TEXT_TERTIARY)
                .add_modifier(Modifier::DIM)),
        ]),
    ];

    let paragraph = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, inner_area);
}
```

---

## Changelog

### 2025-12-30 - Final OLED Tuning
- Brightened `TEXT_TERTIARY` from `RGB(100, 100, 100)` to `RGB(140, 140, 140)` - was unreadable
- Brightened `TEXT_DIM` from `RGB(80, 80, 80)` to `RGB(120, 120, 120)` - footer hints now visible
- Tested on actual OLED display at max brightness
- Works on both pure black and gray terminal themes

### 2025-12-30 - OLED Optimization
- Changed `TEXT_SECONDARY` from `Color::Gray` to `RGB(160, 160, 160)` for better contrast
- Changed `TEXT_TERTIARY` from `Color::DarkGray` to `RGB(100, 100, 100)` for readability
- Added `TEXT_DIM` at `RGB(80, 80, 80)` for very subtle text
- Updated `SEPARATOR` to `RGB(60, 60, 60)` for subtle lines
- Removed all hardcoded `Color::Gray` and `Color::DarkGray` references

### 2025-01-30 - Initial Guide
- Established theme system
- Defined color palette
- Documented component patterns
- Created anti-pattern section

---

## Questions?

When in doubt:
1. Prioritize readability over aesthetics
2. Use more whitespace rather than less
3. Choose dimmer colors for non-critical information
4. Test on actual OLED display
5. Ask: "Would this work in a screenshot?"

For ambiguous cases, refer to modern CLI tools like `gh` (GitHub CLI) or `vercel` for inspiration.
