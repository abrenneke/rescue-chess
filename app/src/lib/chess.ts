export function positionToXy(position: number): [number, number] {
  return [position % 8, Math.floor(position / 8)];
}

export type PieceType = 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King';
export type PawnPromotion = 'Queen' | 'Rook' | 'Bishop' | 'Knight';

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
        value: {
          pos: number;
          promoted_to: PawnPromotion | null;
        };
      }
    | {
        type: 'Capture';
        value: {
          captured: PieceType;
          captured_holding: PieceType | null;
        };
      }
    | {
        type: 'CaptureAndRescue';
        value: {
          captured_type: PieceType;
          rescued_pos: number;
          captured_holding: PieceType | null;
        };
      }
    | {
        type: 'CaptureAndDrop';
        value: {
          captured_type: PieceType;
          drop_pos: number;
          promoted_to: PawnPromotion | null;
          captured_holding: PieceType | null;
        };
      }
    | {
        type: 'EnPassant';
        value: {
          captured_pos: number;
          captured: PieceType;
          captured_holding: PieceType | null;
        };
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
        value: PawnPromotion;
      }
    | {
        type: 'CapturePromotion';
        value: {
          captured: PieceType;
          promoted_to: PawnPromotion;
          captured_holding: PieceType | null;
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

export type WhiteMoveResponse = {
  results: SearchResults;
  move_from_whites_perspective: PieceMove;
};
