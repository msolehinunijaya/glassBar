<script lang="ts">
  // The companion's face: one pair of eyes and an optional mouth, drawn as SVG.
  //
  // SVG and CSS rather than a canvas, deliberately. The eyes are a handful of shapes and the
  // moods are discrete, so moving between them costs nothing while nothing changes -- a canvas
  // would need a frame loop running forever to show the same still face.
  //
  // Chromium treats an SVG rect's height, y and rx as CSS properties, so the eyes blink,
  // squint and widen by transitioning numbers instead of swapping pictures.
  import { LOOKS, type Mood } from './mood';

  let { mood, hands = true, eye = '#ffffff', size = 26 }: { mood: Mood; hands?: boolean; eye?: string; size?: number } = $props();

  /** viewBox units. The eye is always rounded to a capsule, so the radius follows the height. */
  const W = 40, H = 24, EYE_W = 11, CY = 11;
  const look = $derived(LOOKS[mood]);

  // A lid is the eye covered from the top, which is just the rect's own height and y -- no
  // second shape to colour-match against whatever is behind it. The bottom edge stays put, so
  // a heavy lid reads as the eye closing rather than shrinking.
  const shape = (full: number) => {
    const vh = Math.max(0.8, full * (1 - look.lid));
    return { h: vh, y: CY - full / 2 + full * look.lid, rx: Math.min(EYE_W, vh) / 2 };
  };
  const leftEye = $derived(shape(look.h));
  /** The right eye is shut for a wink; everything else is symmetrical. */
  const rightEye = $derived(shape(mood === 'wink' ? 2 : look.h));

  const MOUTHS: Record<string, string> = {
    smile: `M ${W / 2 - 5} 18 Q ${W / 2} 22 ${W / 2 + 5} 18`,
    frown: `M ${W / 2 - 5} 21 Q ${W / 2} 17 ${W / 2 + 5} 21`,
    flat: `M ${W / 2 - 4} 19.5 L ${W / 2 + 4} 19.5`,
  };
  /** A smile-shaped eye: the arc the eyes become when it is pleased. */
  const arc = (cx: number) => `M ${cx - EYE_W / 2} ${CY + 3} Q ${cx} ${CY - 5} ${cx + EYE_W / 2} ${CY + 3}`;
  const cross = (cx: number) => `M ${cx - 4} ${CY - 4} L ${cx + 4} ${CY + 4} M ${cx + 4} ${CY - 4} L ${cx - 4} ${CY + 4}`;
  const heart = (cx: number) =>
    `M ${cx} ${CY + 5} C ${cx - 7} ${CY - 1} ${cx - 5} ${CY - 7} ${cx} ${CY - 3} C ${cx + 5} ${CY - 7} ${cx + 7} ${CY - 1} ${cx} ${CY + 5} Z`;

  const LX = 12, RX = 28;
</script>

<span class="face" class:bob={look.motion === 'bob'} class:shiver={look.motion === 'shiver'} class:sway={look.motion === 'sway'}
      style:--eye={eye} style:--fs="{size}px" aria-hidden="true">
  <svg viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
    {#if look.shape === 'none'}
      <!-- Round eyes. The tilt slants them: inwards glares, outwards droops. -->
      {#each [{ cx: LX, e: leftEye }, { cx: RX, e: rightEye }] as side, i (i)}
        <g class="eyewrap" style:transform="rotate({(i === 0 ? 1 : -1) * look.tilt}deg)" style:transform-origin="{side.cx}px {CY}px">
          <rect class="ball" x={side.cx - EYE_W / 2} width={EYE_W}
                style:y="{side.e.y}px" style:height="{side.e.h}px" style:rx="{side.e.rx}px" />
        </g>
      {/each}
    {:else if look.shape === 'arc'}
      <path class="stroke" d={arc(LX)} />
      <path class="stroke" d={arc(RX)} />
    {:else if look.shape === 'cross'}
      <path class="stroke" d={cross(LX)} />
      <path class="stroke" d={cross(RX)} />
    {:else}
      <path class="ball" d={heart(LX)} />
      <path class="ball" d={heart(RX)} />
    {/if}

    {#if look.mouth === 'open'}
      <ellipse class="ball mouth" cx={W / 2} cy="19" rx="3.2" ry="2.6" />
    {:else if look.mouth !== 'none'}
      <path class="stroke thin" d={MOUTHS[look.mouth]} />
    {/if}

    {#if hands}
      <!-- Two little hands over the pill's bottom edge, so it reads as hanging on to it. -->
      <path class="stroke hand" d="M 4 {H - 1.5} q 2.5 -3 5 0" />
      <path class="stroke hand" d="M {W - 9} {H - 1.5} q 2.5 -3 5 0" />
    {/if}
  </svg>
</span>

<style>
  .face { display: block; flex: none; width: calc(var(--fs) * 40 / 24); height: var(--fs); }
  svg { display: block; width: 100%; height: 100%; overflow: visible; }

  .ball { fill: var(--eye); }
  .stroke { fill: none; stroke: var(--eye); stroke-width: 2.2; stroke-linecap: round; }
  .thin { stroke-width: 1.8; }
  .hand { stroke-width: 2.6; opacity: 0.85; }
  .mouth { opacity: 0.9; }

  /* Blinking, squinting and widening are these numbers moving. Chromium animates an SVG
     rect's geometry as CSS, so there is nothing to redraw by hand. */
  .ball, .eyewrap { transition: height 0.14s var(--ease), y 0.14s var(--ease), rx 0.14s var(--ease), transform 0.22s var(--ease); }

  /* Singing along, being tickled, and swaying half-asleep. All transforms, so none of it
     touches layout. */
  .bob { animation: bob 0.5s ease-in-out infinite; }
  @keyframes bob { 50% { transform: translateY(-1.5px) scale(1.04); } }
  .shiver { animation: shiver 0.12s linear infinite; }
  @keyframes shiver { 25% { transform: translateX(1px) rotate(2deg); } 75% { transform: translateX(-1px) rotate(-2deg); } }
  .sway { animation: sway 2.4s ease-in-out infinite; }
  @keyframes sway { 50% { transform: rotate(4deg); } }
</style>
