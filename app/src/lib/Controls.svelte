<script lang="ts">
  export let onSelfPlayClicked: () => void;
  export let onRestart: () => void;

  export let isSelfPlay;
</script>

<div>
  <div>
    <button type="button" on:click={onSelfPlayClicked}>
      {isSelfPlay ? 'Stop self play' : 'Start self play'}
    </button>
  </div>
  <div>
    <button type="button" on:click={onRestart}> Restart </button>
  </div>
</div>
