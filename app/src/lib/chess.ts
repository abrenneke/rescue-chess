export function positionToXy(position: number): [number, number] {
  return [position % 8, Math.floor(position / 8)];
}

export type PieceMove = {
  from: number;
  to: number;
  move_type: 'Normal' | { Capture: 'Pawn' | 'Knight' | 'Bishop' | 'Rook' | 'Queen' | 'King' };
};

export function isEnum<T>(value: T, enumValue: keyof Extract<T, object>): boolean {
  return value && typeof value === 'object' && enumValue in value;
}
