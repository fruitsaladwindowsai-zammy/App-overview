# Spring & Bounce Physics

Every visual transition in **Winscroller 2d Appview** is driven by real-time discrete spring-damper physics rather than static linear or cubic curves.

![Underdamped Mass-Spring Physics](./assets/concept-spring-physics.svg)

## Mathematical Model

Each frame, `dt = ctx.input(|i| i.stable_dt).clamp(0.001, 0.05)` integrates a unit-mass damped harmonic oscillator:

```rust
let disp = current - target;
let force = -stiffness * disp - damping * velocity;
velocity += force * dt;
current += velocity * dt;
```

## Tuned Production Constants (`src/ui.rs`)

| Animation | Stiffness (`k`) | Damping (`c`) | Behavior |
|---|---|---|---|
| **Entrance & Dismissal** | `220.0` | `17.5` | Underdamped spring: scales grid `0.91 → 1.025 (bounce) → 1.00`. Dismisses cleanly once `anim_progress <= 0.04` and `|velocity| < 0.09`. |
| **Page Snapping** | `180.0` | `19.0` | Smoothly pulls `page_offset` toward `target_page` after two-finger swipes, flings, or arrow-key rollovers. |

## Zero-Waste Background Idle
When `is_visible == false`, `eframe::App::update` skips physics integration and rendering completely, sleeping with `ctx.request_repaint_after(Duration::from_millis(100))` to check channel commands at near-0% CPU.
