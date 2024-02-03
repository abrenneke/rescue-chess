<script>
  let size = 8;
  let board = Array.from({ length: size }, (_, i) => Array.from({ length: size }, (_, j) => ((i + j) % 2 === 0 ? 'white' : 'black')));
</script>

<div class="board">
  {#each board as row, i}
    {#each row as cell, j}
      <div class="cell {cell}" id="{i}-{j}"></div>
    {/each}
  {/each}
</div>

<style>
  .board {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    grid-template-rows: repeat(8, 1fr);
    width: 100vh;
    max-width: 100%;
    max-height: 100%;

    aspect-ratio: 1 / 1;
  }
  .cell {
    border: 1px solid black;
  }
  .white {
    background: #f0d9b5;
  }
  .black {
    background: #b58863;
  }
</style>
