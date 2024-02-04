<script lang="ts">
  import { onMount } from 'svelte';
  import ChessPieces from './ChessPieces.svelte';

  let boardContainer: HTMLDivElement;
  let board: HTMLDivElement;

  let boardGrid = Array.from({ length: 8 }, (_, i) =>
    Array.from({ length: 8 }, (_, j) => ((i + j) % 2 === 0 ? 'white' : 'black')),
  );

  onMount(() => {
    const onResize = () => {
      const containerWidth = boardContainer.clientWidth;

      board.style.height = `${Math.min(containerWidth, window.innerHeight)}px`;
      board.style.width = `${Math.min(containerWidth, window.innerHeight)}px`;
    };

    onResize();

    window.addEventListener('resize', onResize, { passive: true });

    return () => {
      window.removeEventListener('resize', onResize);
    };
  });
</script>

<div class="board-container" bind:this={boardContainer}>
  <div class="board" bind:this={board}>
    {#each boardGrid as row, i}
      {#each row as cell, j}
        <div class="cell {cell}" id="{i}-{j}"></div>
      {/each}
    {/each}
  </div>
  <div class="pieces">
    <ChessPieces {board} />
  </div>
</div>

<style>
  .board-container {
    width: 100%;
    position: relative;
  }

  .board {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    grid-template-rows: repeat(8, 1fr);
  }

  .white {
    background: #f0d9b5;
  }
  .black {
    background: #b58863;
  }
</style>
