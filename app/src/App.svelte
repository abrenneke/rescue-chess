<script lang="ts">
  import ChessBoard from './lib/ChessBoard.svelte';
</script>

<main class="container">
  <div class="board">
    <ChessBoard />
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100%;

    background-color: #e0e5ec;

    display: grid;
    grid-template-columns: 1fr 300px;
  }
</style>
