# DESIGN_RULES.md — Blaze Continuum Design Language (BCDL: Kinetic Glass)

> **Status:** Standardized Specification (v1.0.0)  
> **Target Platforms:** BlazeOS, Niri Wayland Compositor, Linux Layer-Shell, Native Rust/GTK4/Slint UIs  
> **Core Ethos:** Anti-AI-Slop · Optical Rigor · Perceptual Consistency · Zero Layout-Shift

---

## 1. The Anti-AI-Slop Manifesto

Most modern "AI-designed" or futuristic Linux desktop concepts fall victim to aesthetic clichés that degrade usability, exhaust the GPU, and scream amateurism. **BCDL (Blaze Continuum Design Language) forbids these unconditionally.**

### Unconditional Prohibitions (Zero Tolerance)
1. **No Emojis in System Chrome:** System overlays, telemetry pills, and status badges must NEVER use emojis (`⚡`, `🚀`, `🛡️`, `✨`, `🔥`). Use crisp typographic tokens, geometric status dots (`●`, `○`), or monospaced symbols (`::`, `->`, `[!]`).
2. **No Neon Cyberpunk Halos:** Highly saturated cyan/magenta glowing drop shadows, pulsating colored rings, and radial bloom are banned. Shadows must be ambient occlusion or physical refraction only.
3. **No Decorative Full-Window Blurs:** The active/frozen window content (`.jpg` / frame) is sacred. You must NEVER blur the user's code, terminal text, or browser window into an illegible mush. Blur is strictly quarantined to floating HUD controls and window borders.
4. **No Fake Animations:** No indeterminate spinning circles when true progress or state is measurable. State transitions must complete within **120ms** or adhere to critically damped physics.
5. **No Layout-Shift on Numbers:** Numerical readouts (Epoch, PID, memory, latencies, input queues) must NEVER use proportional fonts. All dynamic numbers must use monospaced tabular figures (`tnum`).

---

## 2. The Four Pillars of Kinetic Glass

```
+-------------------------------------------------------------------------+
|                        BCDL: KINETIC GLASS                              |
+-------------------------------------------------------------------------+
|  [PILLAR 1] Material 3 Expressive  -->  8dp Grid & State Opacity Layers |
|  [PILLAR 2] Liquid Glass Restraint -->  1px Specular Hairline & Chrome  |
|  [PILLAR 3] OKLCH Color Space      -->  Algorithmic Contrast (C <= 0.015) |
|  [PILLAR 4] Swiss Typography       -->  Tabular Figures, 0 Layout-Shift |
+-------------------------------------------------------------------------+
```

### Pillar I: Structural Grid & State Layers (M3 Expressive)
* **Base Unit:** 8dp spatial rhythm. Minor paddings may step to 4dp. Margins and container paddings: 8dp, 16dp, 24dp.
* **Deterministic State Opacity:** Do not shift hues or add glowing overlays on state changes. Alter only surface lightness/opacity:
  - **Resting:** Base token opacity (e.g., 72% / 82%).
  - **Hover:** `+4%` white overlay / alpha step.
  - **Active / Pressed:** `+8%` white overlay / alpha step.
  - **Selected / Focused:** `+12%` overlay with 1px mineral accent stroke.

### Pillar II: Material Restraint & Optical Physics (Liquid Glass)
* **Content Preservation:** The captured window buffer (`/tmp/bctp_ghost_surface.jpg`) is placed on the background layer and dimmed by only 10%–15% using `--blz-surface-scrim`. Zero Gaussian blur on window contents.
* **1px Directional Specular Border (Hairline Rim):**
  - **Top & Left Edges:** `oklch(0.95 0.005 255 / 0.16)` (Simulates natural ambient overhead light refracting through bevel).
  - **Bottom & Right Edges:** `oklch(0.95 0.005 255 / 0.05)` (Subtle ground return).
* **Backdrop Blur:** Restricted to the **Telemetry Pill** and modal surfaces. Dual-Kawase blur (radius: 8px to 12px, 3 passes), combined with 92% color saturation pass.

### Pillar III: Perceptual Color Mathematics (OKLCH)
All colors are mathematically derived in **OKLCH** `(Lightness, Chroma, Hue)` to guarantee constant perceptual contrast across high-DPI, HDR, DCI-P3, and sRGB panels:
* **Neutrals (Graphite / Slate):** Chroma clamped to `C <= 0.015`. Prevents cheap tinted plastic appearance.
* **Accent (Mineral Cobalt):** Low-saturation cornflower blue (`C = 0.035`, `H = 255`). Never saturated #00FFFF or #FF00FF.
* **Success / Sync / Alert:** Muted natural earth minerals (Celadon Green `H = 145`, Amber Ochre `H = 75`, Crimson Brick `H = 25`), all with `C <= 0.06`.

### Pillar IV: Industrial Swiss Telemetry & Typography
* **Primary System Face:** Inter Display or system sans-serif (Weights: 400 Regular, 500 Medium, 600 SemiBold).
* **Telemetry & Numeric Readouts:** JetBrains Mono or monospace fallback with mandatory `font-variant-numeric: tabular-nums lining-nums`.
* **Zero Jitter Rule:** When telemetry increments from `1.9ms` to `104.2ms` or `+1 INP` to `+14 INP`, the badge container width MUST NOT shake or jitter. Containers have min-width constraints or tabular spacing.

---

## 3. Official Design Tokens

### 3.1 Color Tokens (OKLCH + sRGB Hex Fallback)

| Token Name | OKLCH Value | Alpha | Hex Fallback | Role / Intended Usage |
| :--- | :--- | :--- | :--- | :--- |
| `--blz-surface-canvas` | `oklch(0.12 0.010 255)` | 1.00 | `#0d0f12` | Deep root workspace / compositor background |
| `--blz-surface-glass-base` | `oklch(0.16 0.012 255)` | 0.72 | `#121519b8` | Primary Layer-Shell HUD & Ghost container |
| `--blz-surface-glass-elevated`| `oklch(0.22 0.015 255)` | 0.82 | `#20252bd1` | Telemetry Pill, active cards, modal dialogs |
| `--blz-surface-scrim` | `oklch(0.10 0.008 255)` | 0.45 | `#08090b73` | Dimmed backdrop over non-focused windows |
| `--blz-border-specular-top` | `oklch(0.95 0.005 255)` | 0.16 | `#ffffff29` | Top/Left 1px inner refractive rim light |
| `--blz-border-specular-bottom`| `oklch(0.95 0.005 255)` | 0.05 | `#ffffff0d` | Bottom/Right 1px inner ambient edge |
| `--blz-accent-mineral` | `oklch(0.82 0.035 255)` | 1.00 | `#a9c7ff` | Active state indicators, focused tabs, primary focus |
| `--blz-status-syncing` | `oklch(0.78 0.050 75)` | 1.00 | `#e5be6b` | Warm amber ochre (Session teleport in-flight) |
| `--blz-status-stable` | `oklch(0.80 0.045 145)` | 1.00 | `#85d79e` | Cool celadon mineral (Commit verified / owner fence) |
| `--blz-status-error` | `oklch(0.68 0.065 25)` | 1.00 | `#e06c75` | Brick crimson (Fencing violation / network fault) |
| `--blz-text-primary` | `oklch(0.96 0.004 255)` | 1.00 | `#e5e9f0` | Window headers, primary telemetry, titles |
| `--blz-text-secondary` | `oklch(0.74 0.010 255)` | 1.00 | `#9ba3af` | Telemetry units (ms, MB), subheaders, labels |
| `--blz-text-muted` | `oklch(0.54 0.012 255)` | 1.00 | `#656c78` | Inactive core IDs, timestamps, grid markings |

---

## 4. Component Anatomy: The Ghost Surface Overlay

The Ghost Surface consists of three distinct layers in the Wayland compositor:

```
[Layer 0: Compositor Desktop (Niri Strip)]
   │
   ▼
[Layer 1: Frozen Content Plane]
   ├── Dimming Scrim: rgba(13, 15, 18, 0.15)
   └── Raw JPEG Buffer: /tmp/bctp_ghost_surface.jpg (Unblurred)
   │
   ▼
[Layer 2: Optical Bezel]
   ├── 1px Asymmetric Specular Border (Top: 0.16, Bottom: 0.05)
   └── Corner Radius: 12px (matches Niri window decoration)
   │
   ▼
[Layer 3: Floating Telemetry Pill (Bottom-Right, 16px offset)]
   ├── Backdrop: --blz-surface-glass-elevated + 10px Kawase Blur
   ├── Border: 1px Specular Refraction
   ├── Pill Content (Tabular figures, JetBrains Mono):
   │     [ ● SYNCING ]  EPOCH #02  ·  1.2ms  ·  +3 INP
   │        ▲              ▲             ▲         ▲
   │        │              │             │         └── Queued input events
   │        │              │             └──────────── T_ghost network latency
   │        │              └────────────────────────── Monotonic Fencing Epoch
   │        └───────────────────────────────────────── Mineral Status Dot
   └── Click/Key Interceptor: Enqueues to InputReplayQueue (Non-blocking)
```

---

## 5. Motion Kinematics & Transition Physics

When interacting with touchpads or animating HUD transitions:
* **Spring Model:** Critically damped spring physics.
  - `stiffness`: `420.0`
  - `damping`: `30.0`
  - `mass`: `1.0`
  - `velocity`: inherited from pointer/gesture vector.
* **Disappearance / Handoff Timing:**
  - Upon BCTP Commit & State Rehydrate, the Ghost Overlay dissolves via `opacity: 1.0 -> 0.0` within exactly **120ms** (`cubic-bezier(0.16, 1, 0.3, 1)`).
  - No slow, lingering fade-outs that block user clicks.

---

## 6. Implementation Checklist for Agents and Engineers

- [ ] Is any window content blurred? **(If yes, REJECT: keep content crisp).**
- [ ] Are there emojis in labels or badges? **(If yes, REJECT: replace with geometric dots or clean text).**
- [ ] Are dynamic numbers using proportional fonts? **(If yes, REJECT: enforce `tabular-nums`).**
- [ ] Are colors glowing neon? **(If yes, REJECT: clamp chroma to `C <= 0.015` or `0.035`).**
- [ ] Does the overlay container shake when numbers increment? **(If yes, REJECT: add tabular spacing / min-width).**
- [ ] Does the overlay block user clicks during transfer? **(If yes, REJECT: route all clicks to `InputReplayQueue`).**
