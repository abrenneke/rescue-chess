<script lang="ts">
  import { onMount } from 'svelte';
  import ChessPiece from './ChessPiece.svelte';
  import { nanoid } from 'nanoid';
  import { invoke } from '@tauri-apps/api/core';
  import PossibleMove from './PossibleMove.svelte';
  import { positionToXy, type PieceMove, isEnum } from './chess';

  type Piece = {
    id: string;
    type: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';
    color: 'white' | 'black';
    x: number;
    y: number;
    displayX: number;
    displayY: number;
  };

  export let board: HTMLDivElement;

  let selectedPiece: Piece | undefined;

  $: selectedPieceType = selectedPiece?.type as Piece['type'];

  let pieces: Piece[] = [];

  let possibleMovePositions: { x: number; y: number; type: 'normal' | 'capture' }[] = [];

  function parseFen(fen: string): Piece[] {
    const [positions] = fen.split(' ');

    const rows = positions.split('/');

    const pieces: Piece[] = [];

    for (let i = 0; i < rows.length; i++) {
      let x = 0;

      for (let j = 0; j < rows[i].length; j++) {
        const char = rows[i][j];

        if (isNaN(parseInt(char))) {
          pieces.push({
            type: char.toLowerCase() as Piece['type'],
            color: char === char.toLowerCase() ? 'black' : 'white',
            x: x,
            y: i,
            id: nanoid(),
            displayX: x,
            displayY: i,
          });

          x++;
        } else {
          x += parseInt(char);
        }
      }
    }

    return pieces;
  }

  async function reloadPieces() {
    const fen = await invoke<string>('get_position_fen', {});
    pieces = parseFen(fen);
  }

  async function applyMove(move: PieceMove) {
    const [fromX, fromY] = positionToXy(move.from);
    const [toX, toY] = positionToXy(move.to);

    await invoke('move_piece', { fromX, fromY, toX, toY });

    const from = pieces.find((p) => p.x === fromX && p.y === fromY)!;

    if (isEnum(move.move_type, 'Capture')) {
      const to = pieces.find((p) => p.x === toX && p.y === toY)!;
      pieces = pieces.filter((p) => p.id !== to.id);
    }

    from.x = toX;
    from.y = toY;

    lerpPieceOverTime(from, [from.displayX, from.displayY], [toX, toY], 150);

    pieces = pieces;
  }

  onMount(async () => {
    await invoke('reset', {});
    await reloadPieces();
  });

  async function onPieceSelected(pieceId: string) {
    const piece = pieces.find((p) => p.id === pieceId);

    if (!piece) {
      return;
    }

    if (piece === selectedPiece) {
      selectedPiece = undefined;
      possibleMovePositions = [];
      return;
    }

    selectedPiece = piece;

    const possibleMoves = await invoke<PieceMove[]>('get_valid_positions_for', {
      x: piece.x,
      y: piece.y,
    });

    possibleMovePositions = possibleMoves.map((move) => {
      const [x, y] = positionToXy(move.to);
      return {
        x,
        y,
        type: move.move_type === 'Normal' ? 'normal' : isEnum(move.move_type, 'Capture') ? 'capture' : 'normal',
      };
    });
  }

  async function onMovePositionSelected(x: number, y: number) {
    if (!selectedPiece) {
      return;
    }

    const possibleMoves = await invoke<PieceMove[]>('get_valid_positions_for', {
      x: selectedPiece.x,
      y: selectedPiece.y,
    });

    const move = possibleMoves.find((move) => {
      const [moveX, moveY] = positionToXy(move.to);
      return moveX === x && moveY === y;
    })!;

    await applyMove(move);

    selectedPiece = undefined;
    possibleMovePositions = [];
  }

  function lerpPiece(piece: Piece, from: [number, number], to: [number, number], t: number) {
    // ease-out
    piece.displayX = from[0] + (to[0] - from[0]) * (1 - Math.pow(1 - t, 2));
    piece.displayY = from[1] + (to[1] - from[1]) * (1 - Math.pow(1 - t, 2));
  }

  function lerpPieceOverTime(piece: Piece, from: [number, number], to: [number, number], duration: number) {
    const start = performance.now();

    function update() {
      const now = performance.now();
      const elapsed = now - start;
      const t = Math.min(1, elapsed / duration);

      lerpPiece(piece, from, to, t);
      pieces = pieces;

      if (t < 1) {
        requestAnimationFrame(update);
      }
    }

    requestAnimationFrame(update);
  }

  $: piecesByColor = pieces.reduce(
    (acc, piece) => {
      if (piece.color === 'white') {
        acc[0].push(piece);
      } else {
        acc[1].push(piece);
      }
      return acc;
    },
    [[], []] as [Piece[], Piece[]],
  );
</script>

<div class="chess-pieces">
  {#each piecesByColor as group}
    {#each group as { x, y, type, color, id, displayX, displayY }}
      <ChessPiece
        {board}
        onSelect={onPieceSelected}
        x={displayX}
        y={displayY}
        {type}
        {color}
        {id}
        isSelected={id === selectedPiece?.id}
      />
    {/each}
  {/each}
  {#each possibleMovePositions as { x, y, type }}
    <PossibleMove {board} {x} {y} pieceType={selectedPieceType} onPositionSelected={onMovePositionSelected} {type} />
  {/each}
</div>

<style>
  .chess-pieces {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }
</style>
