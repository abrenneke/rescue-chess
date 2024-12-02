<script lang="ts">
  import { onMount } from 'svelte';
  import ChessPiece from './ChessPiece.svelte';
  import { nanoid } from 'nanoid';
  import { invoke } from '@tauri-apps/api/core';
  import PossibleMove from './PossibleMove.svelte';
  import { positionToXy, type PieceMove, type BlackMoveResponse } from './chess';
  import { listen } from '@tauri-apps/api/event';

  let blackMoveListener: ((response: BlackMoveResponse) => void) | undefined;

  if (blackMoveListener == null) {
    blackMoveListener = async (response) => {
      console.log('received black move', response);
      await applyMove(response.move_from_whites_perspective);
    };

    listen('black_move', (event) => {
      blackMoveListener!(event.payload as BlackMoveResponse);
    });
  }

  type Piece = {
    id: string;
    type: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';
    color: 'white' | 'black';
    x: number;
    y: number;
    displayX: number;
    displayY: number;
    holding?: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';
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
    await invoke('move_piece', { mv: move });

    applyMoveLocal(move);
  }

  function applyMoveLocal(move: PieceMove) {
    const [fromX, fromY] = positionToXy(move.from);
    const [toX, toY] = positionToXy(move.to);

    const from = pieces.find((p) => p.x === fromX && p.y === fromY);

    if (!from) {
      console.dir(move);
      throw new Error('Piece not found');
    }

    const isCapture =
      move.move_type.type === 'Capture' ||
      move.move_type.type === 'CaptureAndRescue' ||
      move.move_type.type === 'CaptureAndDrop';

    if (isCapture) {
      const to = pieces.find((p) => p.x === toX && p.y === toY)!;
      pieces = pieces.filter((p) => p.id !== to.id);
    }

    if (move.move_type.type === 'NormalAndRescue') {
      const [rescuedX, rescuedY] = positionToXy(move.move_type.value);
      const rescued = pieces.find((p) => p.x === rescuedX && p.y === rescuedY)!;
      pieces = pieces.filter((p) => p.id !== rescued.id);
      from.holding = rescued.type;
    } else if (move.move_type.type === 'CaptureAndRescue') {
      const [rescuedX, rescuedY] = positionToXy(move.move_type.value.rescued_pos);
      const rescued = pieces.find((p) => p.x === rescuedX && p.y === rescuedY)!;
      pieces = pieces.filter((p) => p.id !== rescued.id);
      from.holding = rescued.type;
    } else if (move.move_type.type === 'NormalAndDrop') {
      const [dropX, dropY] = positionToXy(move.move_type.value);
      pieces.push({
        id: nanoid(),
        type: from.holding!,
        color: from.color,
        x: dropX,
        y: dropY,
        displayX: dropX,
        displayY: dropY,
      });
      from.holding = undefined;
    } else if (move.move_type.type === 'CaptureAndDrop') {
      const [dropX, dropY] = positionToXy(move.move_type.value.drop_pos);
      pieces.push({
        id: nanoid(),
        type: from.holding!,
        color: from.color,
        x: dropX,
        y: dropY,
        displayX: dropX,
        displayY: dropY,
      });
      from.holding = undefined;
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
      const isCapture =
        move.move_type.type === 'Capture' ||
        move.move_type.type === 'CaptureAndRescue' ||
        move.move_type.type === 'CaptureAndDrop' ||
        move.move_type.type === 'EnPassant' ||
        move.move_type.type === 'CapturePromotion';

      return {
        x,
        y,
        type: isCapture ? 'capture' : 'normal',
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

    console.log("waiting for black's move");
    await invoke<BlackMoveResponse>('get_black_move', {});
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
    {#each group as { x, y, type, color, id, displayX, displayY, holding }}
      <ChessPiece
        {board}
        onSelect={onPieceSelected}
        x={displayX}
        y={displayY}
        {type}
        {color}
        {id}
        isSelected={id === selectedPiece?.id}
        heldPiece={holding}
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
