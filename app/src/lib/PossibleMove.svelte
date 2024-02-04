<script lang="ts">
  export let board: HTMLDivElement;

  export let x: number;
  export let y: number;

  export let pieceType: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';

  export let type: 'normal' | 'capture';

  export let onPositionSelected: (x: number, y: number) => void;

  let width = board ? board.clientWidth / 8 : 0;
  let height = board ? board.clientHeight / 8 : 0;

  $: left = x * width;
  $: top = y * height;
</script>

<div
  class="possible-move"
  class:normal={type === 'normal'}
  class:capture={type === 'capture'}
  style="left: {left}px; top: {top}px; width: {width}px; height: {height}px;"
  on:click={() => onPositionSelected(x, y)}
>
  <svg>
    <circle class="circle" cx="50%" cy="50%" r="40%" />
  </svg>
</div>

<style>
  .possible-move {
    position: absolute;
    z-index: 1;
    background: transparent;
  }

  .possible-move.capture .circle {
    stroke: rgba(255, 0, 0, 0.5);
  }

  .circle {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background-color: rgba(0, 0, 0, 0.5);
  }

  svg {
    width: 100%;
    height: 100%;
  }

  circle {
    fill: transparent;
    stroke: rgba(0, 0, 0, 0.5);
    stroke-width: 4;
  }
</style>
