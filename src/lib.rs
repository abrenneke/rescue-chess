#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc; // 14% faster on Windows!

pub mod bitboard;
pub mod evaluation;
pub mod piece;
pub mod piece_move;
pub mod pos;
pub mod position;
pub mod search;
pub mod uci;

pub use bitboard::Bitboard;
pub use piece::{Color, Piece, PieceType};
pub use piece_move::PieceMove;
pub use pos::Pos;
pub use position::Position;
