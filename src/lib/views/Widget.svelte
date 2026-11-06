<script lang="ts">
  import { setWindowState, startDrag } from '../services/window';

  let pressed = $state(false);
  let dragging = false;
  let originX = 0;
  let originY = 0;

  const DRAG_THRESHOLD = 5;

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    pressed = true;
    dragging = false;
    originX = event.screenX;
    originY = event.screenY;
  }

  function onPointerMove(event: PointerEvent) {
    if (!pressed || dragging) return;
    const moved = Math.hypot(event.screenX - originX, event.screenY - originY);
    if (moved > DRAG_THRESHOLD) {
      // Se arranca el arrastre nativo en el primer movimiento real, no al pulsar:
      // así un clic limpio sigue siendo un clic.
      dragging = true;
      startDrag().catch(() => {});
    }
  }

  function onPointerUp() {
    if (!pressed) return;
    pressed = false;
    if (!dragging) {
      setWindowState('expanded').catch(() => {});
    }
    dragging = false;
  }
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="stage">
  <button
    class="orb"
    onpointerdown={onPointerDown}
    aria-label="Abrir Asistente DazNova"
    title="Abrir asistente (Alt+Shift+Space)"
  >
    <span class="ring" aria-hidden="true"></span>
    <span class="core" aria-hidden="true"></span>
  </button>
</div>

<style>
  .stage {
    display: grid;
    place-items: center;
    height: 100%;
    padding: 4px;
  }

  .orb {
    position: relative;
    display: grid;
    place-items: center;
    width: 64px;
    height: 64px;
    border-radius: var(--radius-full);
    background: radial-gradient(circle at 50% 40%, #151b28 0%, #080b12 66%, #04060a 100%);
    box-shadow:
      0 8px 22px rgba(0, 0, 0, 0.55),
      inset 0 0 0 1px rgba(255, 255, 255, 0.06);
    cursor: grab;
    transition:
      transform var(--dur-fast) var(--ease-out),
      box-shadow var(--dur-normal) var(--ease-out);
  }

  .orb:hover {
    transform: scale(1.04);
    box-shadow:
      0 10px 26px rgba(0, 0, 0, 0.6),
      0 0 20px rgba(76, 201, 240, 0.35),
      inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  }

  .orb:active {
    cursor: grabbing;
    transform: scale(0.97);
  }

  .ring {
    position: absolute;
    inset: 5px;
    border-radius: var(--radius-full);
    background: conic-gradient(
      from 140deg,
      var(--accent-cyan),
      var(--accent-violet),
      var(--accent-magenta),
      var(--accent-cyan)
    );
    -webkit-mask: radial-gradient(circle, transparent 63%, #000 64%);
    mask: radial-gradient(circle, transparent 63%, #000 64%);
    animation: orbit 16s linear infinite;
  }

  .core {
    width: 13px;
    height: 13px;
    border-radius: var(--radius-full);
    background: var(--text-primary);
    box-shadow:
      0 0 12px rgba(76, 201, 240, 0.9),
      0 0 3px rgba(255, 255, 255, 0.9);
  }

  @keyframes orbit {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .ring {
      animation: none;
    }
  }
</style>
