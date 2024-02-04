<script lang="ts">
  import { onMount } from 'svelte';
  import chessPiecesSvg from './chess-pieces.svg';

  export let board: HTMLDivElement;
  export let onDragStart: (event: DragEvent) => void;

  let piece: HTMLDivElement;

  export let color: string;
  export let type: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';

  export let x: number;
  export let y: number;

  export let ghostX: number;
  export let ghostY: number;

  export let isDragging: boolean;

  export let id: string;

  let width = 0;
  let height = 0;

  $: left = x * width;
  $: top = y * height;

  $: ghostLeft = ghostX * width;
  $: ghostTop = ghostY * height;

  $: viewBoxX = {
    k: 0,
    q: 45,
    b: 90,
    n: 135,
    r: 180,
    p: 225,
  }[type];
  $: viewBoxY = color === 'black' ? 45 : 0;

  onMount(() => {
    const resize = () => {
      if (!board) {
        return;
      }

      width = board.clientWidth / 8;
      height = board.clientHeight / 8;
    };

    resize();

    window.addEventListener('resize', resize, { passive: true });

    return () => {
      window.removeEventListener('resize', resize);
    };
  });
</script>

<div
  class="chess-piece"
  class:is-dragging={isDragging}
  bind:this={piece}
  draggable="true"
  on:dragstart={onDragStart}
  style="width: {width}px; height: {height}px; left: {left}px; top: {top}px;"
  data-id={id}
>
  {#if isDragging}
    <div class="dragging-ghost" style="left: {ghostLeft}px; top: {ghostTop}px">
      <img class="icon" alt="chess piece" src={`${chessPiecesSvg}#svgView(viewBox(${viewBoxX} ${viewBoxY} 45 45))`} />
    </div>
  {/if}
  <img class="icon" alt="chess piece" src={`${chessPiecesSvg}#svgView(viewBox(${viewBoxX} ${viewBoxY} 45 45))`} />
</div>

<style>
  .chess-piece {
    position: absolute;
  }

  .chess-piece.is-dragging {
    & > .icon {
      opacity: 0.5;
    }
  }

  .dragging-ghost {
    position: absolute;
    background: transparent;
    transition:
      left 0.1s ease-out,
      top 0.1s ease-out;

    width: 100%;
    height: 100%;
  }

  .icon {
    width: 100%;
    height: 100%;
  }
</style>
