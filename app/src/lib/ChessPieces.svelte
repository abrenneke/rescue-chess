<script lang="ts">
  import { onMount } from 'svelte';
  import ChessPiece from './ChessPiece.svelte';
  import { nanoid } from 'nanoid';

  export let fen: string;

  type Piece = {
    id: string;
    type: 'k' | 'q' | 'b' | 'n' | 'r' | 'p';
    color: 'white' | 'black';
    x: number;
    y: number;
    ghostX: number;
    ghostY: number;
  };

  export let board: HTMLDivElement;

  let draggingPiece: { id: string; element: HTMLDivElement } | undefined;

  let pieces: Piece[] = [];

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
            ghostX: 0,
            ghostY: 0,
            id: nanoid(),
          });

          x++;
        } else {
          x += parseInt(char);
        }
      }
    }

    return pieces;
  }

  onMount(() => {
    pieces = parseFen(fen);
  });

  function dragStart(event: DragEvent) {
    const element = event.currentTarget as HTMLDivElement;

    draggingPiece = {
      id: element.dataset.id!,
      element,
    };

    document.addEventListener('mousemove', dragPiece);
    document.addEventListener('mouseup', dropPiece);
  }

  function dragPiece(event: MouseEvent) {
    if (draggingPiece == null) {
      return;
    }

    const piece = pieces.find((piece) => piece.id === draggingPiece!.id)!;

    const mousePositionRelativeToBoard = {
      x: event.clientX - board.getBoundingClientRect().left,
      y: event.clientY - board.getBoundingClientRect().top,
    };

    const closestCell = {
      x: Math.floor(mousePositionRelativeToBoard.x / (board.clientWidth / 8)),
      y: Math.floor(mousePositionRelativeToBoard.y / (board.clientHeight / 8)),
    };

    if (closestCell.x < 0 || closestCell.x > 7 || closestCell.y < 0 || closestCell.y > 7) {
      return;
    }

    const relativeCell = {
      x: closestCell.x - piece.x,
      y: closestCell.y - piece.y,
    };

    piece.ghostX = relativeCell.x;
    piece.ghostY = relativeCell.y;

    pieces = pieces;
  }

  function dropPiece() {
    if (draggingPiece) {
      const piece = pieces.find((piece) => piece.id === draggingPiece!.id)!;

      piece.x += piece.ghostX;
      piece.y += piece.ghostY;

      piece.ghostX = 0;
      piece.ghostY = 0;

      draggingPiece = undefined;
      document.removeEventListener('mousemove', dragPiece);

      pieces = [...pieces];
    }
  }
</script>

<div
  class="chess-pieces"
  on:drop={dropPiece}
  on:dragstart={(e) => {
    e.preventDefault();
  }}
  on:dragover={(e) => {
    e.preventDefault();
  }}
>
  {#each pieces as { x, y, ghostX, ghostY, type, color, id }}
    <ChessPiece
      {board}
      onDragStart={dragStart}
      {x}
      {y}
      {ghostX}
      {ghostY}
      {type}
      {color}
      {id}
      isDragging={id === draggingPiece?.id}
    />
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
