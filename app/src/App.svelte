<script lang="ts">
  import ChessBoard from './lib/ChessBoard.svelte';
  import Controls from './lib/Controls.svelte';

  let isSelfPlay = false;

  function onSelfPlayClicked() {
    isSelfPlay = !isSelfPlay;
  }

  function onRestart() {
    window.location.reload();
  }
</script>

<main class="container">
  <div class="board">
    <ChessBoard {isSelfPlay} />
  </div>
  <div class="controls">
    <Controls {isSelfPlay} {onSelfPlayClicked} {onRestart} />
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100%;

    background-color: #e0e5ec;

    display: grid;
    grid-template-columns: 1fr 200px;
  }

  .board {
    display: flex;
    align-items: center;
    max-height: 100%;
    overflow: hidden;
  }
</style>
