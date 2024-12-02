<script lang="ts">
  import { onMount } from 'svelte';
  import chessPiecesSvg from './chess-pieces.svg';

  export let board: HTMLDivElement;
  export let onSelect: (id: string) => void;

  let piece: HTMLDivElement;

  export let color: string;
  export let type: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';

  // Update type to only include the piece type since color must match
  export let heldPiece: 'k' | 'q' | 'b' | 'n' | 'r' | 'p' | null = null;

  export let x: number;
  export let y: number;

  export let isSelected: boolean;

  export let id: string;

  let width = 0;
  let height = 0;

  $: left = x * width;
  $: top = y * height;

  $: viewBoxX = {
    k: 0,
    q: 45,
    b: 90,
    n: 135,
    r: 180,
    p: 225,
  }[type];
  $: viewBoxY = color === 'black' ? 45 : 0;

  // Held piece uses same color as main piece
  $: heldPieceViewBoxX = heldPiece
    ? {
        k: 0,
        q: 45,
        b: 90,
        n: 135,
        r: 180,
        p: 225,
      }[heldPiece]
    : 0;
  $: heldPieceViewBoxY = viewBoxY; // Same color as parent piece

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
  class:is-selected={isSelected}
  bind:this={piece}
  on:mousedown={() => onSelect(id)}
  style="width: {width}px; height: {height}px; left: {left}px; top: {top}px;"
  data-id={id}
>
  <div class="selected-border" />
  <img class="icon" alt="chess piece" src={`${chessPiecesSvg}#svgView(viewBox(${viewBoxX} ${viewBoxY} 45 45))`} />

  {#if heldPiece}
    <div class="held-piece">
      <img
        class="held-piece-icon"
        alt="held chess piece"
        src={`${chessPiecesSvg}#svgView(viewBox(${heldPieceViewBoxX} ${heldPieceViewBoxY} 45 45))`}
      />
    </div>
  {/if}
</div>

<style>
  .chess-piece {
    position: absolute;
  }

  .selected-border {
    display: none;
    position: absolute;
    width: 100%;
    height: 100%;
    border: 4px solid rgba(255, 0, 0, 0.5);
  }

  .chess-piece.is-selected > .selected-border {
    display: block;
  }

  .icon {
    width: 100%;
    height: 100%;
  }

  .held-piece {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 40%;
    height: 40%;
    background: rgba(255, 255, 255, 0.8);
    border-radius: 50%;
    padding: 2px;
  }

  .held-piece-icon {
    width: 100%;
    height: 100%;
  }
</style>
