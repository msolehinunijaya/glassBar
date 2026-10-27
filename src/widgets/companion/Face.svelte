<script lang="ts">
  // The companion's face: one pair of eyes and an optional mouth, drawn as SVG.
  //
  // SVG and CSS rather than a canvas, deliberately. The eyes are a handful of shapes and the
  // moods are discrete, so moving between them costs nothing while nothing changes -- a canvas
  // would need a frame loop running forever to show the same still face.
  //
  // Everything moves by `transform` only. SVG geometry (y, height, rx) can be animated as CSS
  // in some browsers and silently can't in others, and an eye that doesn't move is a dead
  // face; a transform is animatable everywhere and stays on the compositor.
  import { LOOKS, type Mood } from './mood';

  let { mood, hands = true, eye = '#ffffff', size = 26 }: { mood: Mood; hands?: boolean; eye?: string; size?: number } = $props();

  /** viewBox units. The eye is drawn once, at full size, and scaled from there. */
  const W = 40, H = 24, EYE = 11, CY = 11, LX = 12, RX = 28;
  const look = $derived(LOOKS[mood]);

  /**
   * How open an eye is, as a vertical scale, and how far the lid has pushed it down.
   *
   * A lid is the eye squashed and sitting lower, which is what a half-closed eye looks like
   * and needs no second shape to colour-match against whatever is behind it.
   */
  const openness = (h: number) => (h / EYE) * (1 - look.lid);
  const drop = $derived(look.lid * 3);
  const leftOpen = $derived(openness(look.h));
  /** The right eye is shut for a wink; everything else is symmetrical. */
  const rightOpen = $derived(openness(mood === 'wink' ? 2 : look.h));

  const MOUTHS: Record<string, string> = {
    smile: `M ${W / 2 - 5} 18 Q ${W / 2} 22 ${W / 2 + 5} 18`,
    frown: `M ${W / 2 - 5} 21 Q ${W / 2} 17 ${W / 2 + 5} 21`,
    flat: `M ${W / 2 - 4} 19.5 L ${W / 2 + 4} 19.5`,
  };
  /** The smile-shaped eye it gets when pleased. */
  const arc = (cx: number) => `M ${cx - EYE / 2} ${CY + 3} Q ${cx} ${CY - 5} ${cx + EYE / 2} ${CY + 3}`;
  const cross = (cx: number) => `M ${cx - 4} ${CY - 4} L ${cx + 4} ${CY + 4} M ${cx + 4} ${CY - 4} L ${cx - 4} ${CY + 4}`;
  const heart = (cx: number) =>
    `M ${cx} ${CY + 5} C ${cx - 7} ${CY - 1} ${cx - 5} ${CY - 7} ${cx} ${CY - 3} C ${cx + 5} ${CY - 7} ${cx + 7} ${CY - 1} ${cx} ${CY + 5} Z`;
</script>

<span class="face" class:bob={look.motion === 'bob'} class:shiver={look.motion === 'shiver'} class:sway={look.motion === 'sway'}
      style:--eye={eye} style:--fs="{size}px" aria-hidden="true">
  <svg viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
    {#if look.shape === 'none'}
      <!-- Round eyes. The tilt slants them: inwards glares, outwards droops. -->
      {#each [{ cx: LX, open: leftOpen, sign: 1 }, { cx: RX, open: rightOpen, sign: -1 }] as side, i (i)}
        <g class="eyewrap" style:transform="rotate({side.sign * look.tilt}deg)" style:transform-origin="{side.cx}px {CY}px">
          <rect class="ball" x={side.cx - EYE / 2} y={CY - EYE / 2} width={EYE} height={EYE} rx={EYE / 2}
                style:transform="translateY({drop}px) scaleY({side.open})" style:transform-origin="{side.cx}px {CY}px" />
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
      <!-- Two little hands at the pill's bottom edge, so it reads as hanging on to it. -->
      <path class="stroke hand" d="M 4 {H - 1.5} q 2.5 -3 5 0" />
      <path class="stroke hand" d="M {W - 9} {H - 1.5} q 2.5 -3 5 0" />
    {/if}
  </svg>
</span>

<style>
  /* max-width matters on a side dock: the column is 56-96 px, so a large face would overflow
     it. The viewBox letterboxes instead of distorting, so it just comes out smaller. */
  .face { display: block; flex: none; width: calc(var(--fs) * 40 / 24); max-width: 100%; height: var(--fs); }
  svg { display: block; width: 100%; height: 100%; overflow: visible; }

  .ball { fill: var(--eye); }
  .stroke { fill: none; stroke: var(--eye); stroke-width: 2.2; stroke-linecap: round; }
  .thin { stroke-width: 1.8; }
  .hand { stroke-width: 2.6; opacity: 0.85; }
  .mouth { opacity: 0.9; }

  /* Blinking, squinting and widening are all this one transform moving. */
  .ball { transition: transform 0.14s var(--ease); }
  .eyewrap { transition: transform 0.22s var(--ease); }

  /* Singing along, being tickled, and swaying half-asleep -- transforms, so none of it
     touches layout. The global reduced-motion rules in glass.css stop them. */
  .bob { animation: bob 0.5s ease-in-out infinite; }
  @keyframes bob { 50% { transform: translateY(-1.5px) scale(1.04); } }
  .shiver { animation: shiver 0.12s linear infinite; }
  @keyframes shiver { 25% { transform: translateX(1px) rotate(2deg); } 75% { transform: translateX(-1px) rotate(-2deg); } }
  /* A few swings, then still: an endless sway kept the bar redrawing all the time it slept. */
  .sway { animation: sway 2.4s ease-in-out 3; }
  @keyframes sway { 50% { transform: rotate(4deg); } }
  :global(:root[data-perf='low']) .face { animation: none; }
  :global(:root[data-perf='high']) .sway { animation-iteration-count: infinite; }
</style>
