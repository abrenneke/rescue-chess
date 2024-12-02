<script lang="ts">
  import { onMount } from 'svelte';

  export let board: HTMLDivElement;
  export let fromX: number;
  export let fromY: number;
  export let toX: number;
  export let toY: number;

  // New props for rescue/drop actions
  export let rescueFromX: number | undefined = undefined;
  export let rescueFromY: number | undefined = undefined;
  export let dropToX: number | undefined = undefined;
  export let dropToY: number | undefined = undefined;

  let width = 0;
  let height = 0;

  $: squareWidth = width / 8;
  $: squareHeight = height / 8;

  function calculateArrowPoints(startX: number, startY: number, endX: number, endY: number) {
    const dx = endX - startX;
    const dy = endY - startY;
    const length = Math.sqrt(dx * dx + dy * dy);

    // Shorten the line slightly to make room for the arrowhead
    const shortenBy = Math.min(20, length * 0.2);
    const ratio = (length - shortenBy) / length;

    const adjustedEndX = startX + dx * ratio;
    const adjustedEndY = startY + dy * ratio;

    const angle = Math.atan2(endY - startY, endX - startX) * (180 / Math.PI);

    return {
      startX,
      startY,
      endX: adjustedEndX,
      endY: adjustedEndY,
      angle,
    };
  }

  // Calculate center points and angles for main move arrow
  $: mainArrow = calculateArrowPoints(
    (fromX + 0.5) * squareWidth,
    (fromY + 0.5) * squareHeight,
    (toX + 0.5) * squareWidth,
    (toY + 0.5) * squareHeight,
  );

  // Calculate rescue arrow points if rescue coordinates are provided
  $: rescueArrow =
    rescueFromX !== undefined && rescueFromY !== undefined
      ? calculateArrowPoints(
          (toX + 0.5) * squareWidth,
          (toY + 0.5) * squareHeight,
          (rescueFromX + 0.5) * squareWidth,
          (rescueFromY + 0.5) * squareHeight,
        )
      : undefined;

  // Calculate drop arrow points if drop coordinates are provided
  $: dropArrow =
    dropToX !== undefined && dropToY !== undefined
      ? calculateArrowPoints(
          (toX + 0.5) * squareWidth,
          (toY + 0.5) * squareHeight,
          (dropToX + 0.5) * squareWidth,
          (dropToY + 0.5) * squareHeight,
        )
      : undefined;

  onMount(() => {
    const resize = () => {
      if (!board) return;
      width = board.clientWidth;
      height = board.clientHeight;
    };

    resize();
    window.addEventListener('resize', resize, { passive: true });

    return () => {
      window.removeEventListener('resize', resize);
    };
  });
</script>

<div class="arrow-container" style="width: {width}px; height: {height}px;">
  <svg width="100%" height="100%">
    <!-- Main move arrow (orange) -->
    <line x1={mainArrow.startX} y1={mainArrow.startY} x2={mainArrow.endX} y2={mainArrow.endY} class="main-arrow" />
    <polygon
      points="0,-6 12,0 0,6"
      class="main-arrow"
      transform="translate({mainArrow.endX},{mainArrow.endY}) rotate({mainArrow.angle})"
    />

    <!-- Rescue arrow (blue) -->
    {#if rescueArrow}
      <line
        x1={rescueArrow.startX}
        y1={rescueArrow.startY}
        x2={rescueArrow.endX}
        y2={rescueArrow.endY}
        class="rescue-arrow"
      />
      <polygon
        points="0,-6 12,0 0,6"
        class="rescue-arrow"
        transform="translate({rescueArrow.endX},{rescueArrow.endY}) rotate({rescueArrow.angle})"
      />
    {/if}

    <!-- Drop arrow (green) -->
    {#if dropArrow}
      <line x1={dropArrow.startX} y1={dropArrow.startY} x2={dropArrow.endX} y2={dropArrow.endY} class="drop-arrow" />
      <polygon
        points="0,-6 12,0 0,6"
        class="drop-arrow"
        transform="translate({dropArrow.endX},{dropArrow.endY}) rotate({dropArrow.angle})"
      />
    {/if}
  </svg>
</div>

<style>
  .arrow-container {
    position: absolute;
    top: 0;
    left: 0;
    pointer-events: none;
  }

  :global(.main-arrow) {
    stroke: rgba(255, 128, 0, 0.7);
    fill: rgba(255, 128, 0, 0.7);
    stroke-width: 4;
  }

  :global(.rescue-arrow) {
    stroke: rgba(0, 128, 255, 0.7);
    fill: rgba(0, 128, 255, 0.7);
    stroke-width: 4;
  }

  :global(.drop-arrow) {
    stroke: rgba(0, 255, 128, 0.7);
    fill: rgba(0, 255, 128, 0.7);
    stroke-width: 4;
  }
</style>
