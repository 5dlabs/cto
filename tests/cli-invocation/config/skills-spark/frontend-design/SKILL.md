---
name: frontend-design
description: Create distinctive, production-grade frontend interfaces with high design quality. Use when building UI components, pages, or applications to avoid generic AI aesthetics.
agents: [blaze, tap, spark]
triggers: [design, ui, aesthetic, beautiful, distinctive, frontend, component, page]
---

# Frontend Design Excellence

Create distinctive, production-grade frontend interfaces that avoid generic "AI slop" aesthetics.

## Core Philosophy

**Bold intentionality beats safe defaults.**

The key is making a clear conceptual choice and executing it with precision. Both maximalism and minimalism work—the key is intentionality, not intensity.

---

## Design Thinking Process

Before coding, commit to a BOLD aesthetic direction:

### 1. Understand Context
- **Purpose**: What problem does this interface solve?
- **Audience**: Who uses it? What do they expect?
- **Constraints**: Performance, accessibility, framework requirements

### 2. Choose an Aesthetic Direction

Pick an EXTREME, not a middle ground:

| Direction | Characteristics |
|-----------|-----------------|
| Brutally minimal | Sparse, lots of white space, typography-focused |
| Maximalist chaos | Layered, dense, many visual elements |
| Retro-futuristic | Neon, grids, tech-inspired |
| Organic/natural | Flowing shapes, earth tones, texture |
| Luxury/refined | Elegant typography, subtle animations, premium feel |
| Playful/toy-like | Bright colors, rounded shapes, bouncy animations |
| Editorial/magazine | Strong typography hierarchy, dramatic imagery |
| Brutalist/raw | Exposed structure, monospace fonts, raw aesthetic |
| Art deco/geometric | Angular shapes, gold accents, symmetry |
| Industrial/utilitarian | Functional, monochrome, purposeful |

### 3. Define the Memorable Element
What's the ONE thing someone will remember about this interface?

---

## What to NEVER Do (AI Slop Indicators)

### Generic Font Choices
| Avoid | Why |
|-------|-----|
| Inter | Overused default, signals "I didn't think about fonts" |
| Roboto | Same problem |
| Arial | Generic system font |
| System fonts stack | Lazy default |
| Space Grotesk | Becoming the new "AI default" |

**Instead:** Choose distinctive, characterful fonts. Pair a unique display font with a refined body font.

### Clichéd Color Schemes
| Avoid | Why |
|-------|-----|
| Purple gradients on white | THE classic AI aesthetic |
| Blue-to-purple gradients | Overused in tech |
| Generic "startup" palettes | Teal + coral + white |
| Evenly distributed pastel palettes | Timid, no hierarchy |

**Instead:** Commit to a dominant color with sharp accents. Create visual hierarchy through color weight.

### Predictable Layouts
| Avoid | Why |
|-------|-----|
| Center-aligned everything | Signals "I used a template" |
| Symmetric grid | Safe but forgettable |
| Hero + 3-column features + footer | The AI landing page |
| Cookie-cutter card grids | Generic pattern |

**Instead:** Use asymmetry, overlap, diagonal flow, grid-breaking elements, or generous negative space.

---

## What to DO

### Typography
- Choose fonts that are beautiful, unique, and interesting
- Pair a distinctive display font with a refined body font
- Create strong typographic hierarchy
- Consider custom letter-spacing and line-height

### Color & Theme
- Commit to a cohesive aesthetic
- Use CSS variables for consistency
- Dominant colors with sharp accents > timid, even palettes
- Consider dark/light as a design choice, not just a toggle

### Motion & Animation
- Focus on high-impact moments: one well-orchestrated page load > scattered micro-interactions
- Use staggered reveals with `animation-delay`
- Scroll-triggered animations that surprise
- Hover states that delight
- Prefer CSS-only for HTML; use Motion/Framer for React

### Spatial Composition
- Embrace asymmetry
- Use overlap and layering
- Consider diagonal flow
- Add grid-breaking elements
- Generous negative space OR controlled density (choose one)

### Backgrounds & Details
Create atmosphere rather than defaulting to solid colors:
- Gradient meshes
- Noise textures
- Geometric patterns
- Layered transparencies
- Dramatic shadows
- Decorative borders
- Custom cursors
- Grain overlays

---

## Implementation Complexity

**Match implementation to aesthetic vision:**

| Aesthetic | Implementation |
|-----------|----------------|
| Maximalist/dramatic | Elaborate code, extensive animations, many effects |
| Minimalist/refined | Restraint, precision, careful spacing, subtle details |

Elegance comes from executing the vision well, not from complexity itself.

---

## Quick Reference Checklist

Before submitting UI code:

### Fonts
- [ ] NOT using Inter, Roboto, Arial, or Space Grotesk
- [ ] Font choice is intentional and distinctive
- [ ] Typography hierarchy is clear

### Colors
- [ ] NOT using purple gradient on white
- [ ] Color scheme has clear hierarchy (dominant + accent)
- [ ] Theme is cohesive and intentional

### Layout
- [ ] NOT using generic center-aligned template
- [ ] Layout has visual interest (asymmetry, overlap, or intentional symmetry)
- [ ] Spacing is generous OR dense (not timid)

### Polish
- [ ] At least one memorable animation or interaction
- [ ] Background has depth (not just solid color)
- [ ] Details match the chosen aesthetic direction

---

## The Bottom Line

> "Claude is capable of extraordinary creative work. Don't hold back—show what can truly be created when thinking outside the box and committing fully to a distinctive vision."

No design should be the same. Vary between light and dark, different fonts, different aesthetics. NEVER converge on common choices across generations.
