export function positionToXy(position: number): [number, number] {
  return [position % 8, Math.floor(position / 8)];
}

export type PieceMove = {
  from: number;
  to: number;
  move_type:
    | {
        type: 'Unknown';
      }
    | {
        type: 'Normal';
      }
    | {
        type: 'NormalAndRescue';
        value: number;
      }
    | {
        type: 'NormalAndDrop';
        value: number;
      }
    | {
        type: 'Capture';
        value: 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King';
      }
    | {
        type: 'CaptureAndRescue';
        value: {
          captured_type: 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King';
          rescued_pos: number;
        };
      }
    | {
        type: 'CaptureAndDrop';
        value: {
          captured_type: 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King';
          drop_pos: number;
        };
      }
    | {
        type: 'EnPassant';
        value: number;
      }
    | {
        type: 'Castle';
        value: {
          king: number;
          rook: number;
        };
      }
    | {
        type: 'Promotion';
        value: 'Queen' | 'Rook' | 'Bishop' | 'Knight';
      }
    | {
        type: 'CapturePromotion';
        value: {
          captured: 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King';
          promoted_to: 'Queen' | 'Rook' | 'Bishop' | 'Knight';
        };
      };
};

export type SearchResults = {
  best_move: PieceMove;
  score: number;
  nodes_searched: number;
  depth: number;
  time_taken_ms: number;
};

export type BlackMoveResponse = {
  results: SearchResults;
  move_from_whites_perspective: PieceMove;
};
