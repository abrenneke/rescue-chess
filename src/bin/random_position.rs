use rand::Rng;
use rescue_chess::{piece_move::GameType, Color, Piece, PieceType, Pos, Position};

fn main() {
    let rng = rand::thread_rng();

    let position = random_non_checkmate_board(rng).unwrap();

    println!("{}", position.to_fen());
}

fn random_non_checkmate_board(mut rng: impl Rng) -> Result<Position, anyhow::Error> {
    let game_type = GameType::Rescue;
    let mut position = random_board(&mut rng);
    while position.is_checkmate(game_type)? || position.inverted().is_checkmate(game_type)? {
        position = random_board(&mut rng);
    }
    Ok(position)
}

fn random_board(mut rng: impl Rng) -> Position {
    let mut start_position = Position::start_position();
    let mut destination_position: Position = "8/8/8/8/8/8/8/8".into();

    // For each piece, place it in a random position on the board.
    // It doesn't matter if the pieces clobber each other, as it just means those pieces have been captured.
    for piece in start_position
        .white_pieces
        .iter_mut()
        .filter(|p| p.piece_type != PieceType::King)
    {
        piece.position = Pos::xy(rng.gen_range(0..8), rng.gen_range(0..8));
    }

    for piece in start_position
        .black_pieces
        .iter_mut()
        .filter(|p| p.piece_type != PieceType::King)
    {
        piece.position = Pos::xy(rng.gen_range(0..8), rng.gen_range(0..8));
    }

    start_position.white_pieces.push(Piece::new(
        PieceType::King,
        Color::White,
        Pos::xy(rng.gen_range(0..8), rng.gen_range(0..8)),
    ));
    start_position.black_pieces.push(Piece::new(
        PieceType::King,
        Color::Black,
        Pos::xy(rng.gen_range(0..8), rng.gen_range(0..8)),
    ));

    destination_position.white_pieces = start_position.white_pieces;
    destination_position.black_pieces = start_position.black_pieces;
    destination_position.calc_changes();

    destination_position
}
