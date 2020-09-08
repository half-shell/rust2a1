use std::cmp;
use std::fmt;

const BOARD_SIZE: usize = 8;
const FILES: [char; BOARD_SIZE] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

#[derive(Debug)]
enum Error {
    PositionCannotBeParsed,
    IllegalMove,
}

#[derive(Debug, Copy, Clone)]
struct Position {
    x: u8, // FILE
    y: u8, // RANK
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            // NOTE(brick): https://stackoverflow.com/questions/44552219/how-to-multiply-divide-add-subtract-numbers-of-different-types
            FILES[std::mem::size_of::<char>() * self.x as usize],
            self.y - 1
        )
    }
}

impl cmp::PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        if self.x == other.x && self.y == other.y {
            return true;
        } else {
            return false;
        }
    }
}

fn string_to_position(coordinates: &str) -> std::result::Result<Position, Error> {
    // TODO(brick): Handle exceptions when parsing string
    // This should also be able to take into account annotation for pieces taken,
    // castling, promotion...
    let (sfile, srank) = coordinates.split_at(1);

    // NOTE(brick): Definitely candidate for refactoring
    let file = match FILES.iter().position(|f| f.to_string() == sfile) {
        Some(position) => position as u8 + 1,
        _ => 0,
    };

    let rank = match u8::from_str_radix(srank, 10) {
        Ok(rank) => {
            if rank <= 0 || rank > 8 {
                0
            } else {
                rank
            }
        }
        _ => 0,
    };

    if rank == 0 || file == 0 {
        return Err(Error::PositionCannotBeParsed);
    };

    Ok(Position {
        x: file as u8,
        y: rank,
    })
}

#[derive(Debug, Copy, Clone)]
struct Piece {
    piece_type: PieceType,
    position: Position,
    side: Player,
}

#[derive(Debug, Copy, Clone)]
struct Move {
    from: Position,
    to: Position,
    // NOTE(brick): This could be an `Option` or even removed in the first iteration
    // since we don't know the Piece moved before knowing the `from` param
    piece_type: PieceType,
    capture: bool,
    rock: bool,
}

struct Game {
    player: Player,
    moves: Vec<Move>, // NOTE(brick): Would we be able to simplify this struct?
    board: Vec<Piece>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{} {}",
            self.moves[0].piece_type, self.moves[0].from, self.moves[0].to
        )
    }
}

impl Game {
    fn move_piece(&mut self, from: Position, to: Position) -> &mut Self {
        let piece_idx = self
            .board
            .iter()
            .position(|piece| piece.position.x == from.x && piece.position.y == from.y);

        match piece_idx {
            Some(idx) => {
                let capture_piece_idx = self
                    .board
                    .iter()
                    .position(|piece| piece.position.x == to.x && piece.position.y == to.y);

                // IF ROCK
                if self.board[idx].piece_type == PieceType::King
                    && from.x == to.x
                    && (from.y + 2 == to.y || from.y - 2 == to.y)
                {
                    self.board[idx].position = to;

                    let good_side_rook_idx = self.board.iter().position(|piece| {
                        piece.position.x == self.board[idx].position.x
                            && piece.piece_type == PieceType::Rook
                            && piece.position.y == (|| if to.y == 2 { 1 } else { 8 })()
                    });
                    
                    match good_side_rook_idx {
                        Some(rook_idx) => {
                            self.board[rook_idx].position.y = (|| if to.y == 2 { 3 } else { 5 })();
                            println!("{:#?}", self.board[rook_idx]);
                        }
                        None => panic!(String::from("No rook placed")),
                    };

                    let new_move = Move {
                        from: from,
                        to: to,
                        piece_type: self.board[idx].piece_type,
                        capture: false,
                        rock: true,
                    };

                    self.moves.push(new_move);
                }

                // IF CAPTURE
                match capture_piece_idx {
                    Some(captured_idx) => {
                        // Is the same player side
                        if self.board[captured_idx].side == self.board[idx].side {
                            panic!(String::from("Illegal move on same player's piece"))
                        }

                        self.board[idx].position = to;
                        self.board.remove(captured_idx);

                        let new_move = Move {
                            from: from,
                            to: to,
                            piece_type: self.board[idx].piece_type,
                            capture: true,
                            rock: false,
                        };

                        self.moves.push(new_move);
                    }

                    // If is just a move
                    None => {
                        self.board[idx].position = to;

                        let new_move = Move {
                            from: from,
                            to: to,
                            piece_type: self.board[idx].piece_type,
                            capture: false,
                            rock: false,
                        };

                        self.moves.push(new_move);
                    }
                }

                return self;
            }
            // TODO(brick): This probably shouldn't panic, but either return an error code
            // or ignore the move all together
            None => panic!(String::from("No piece was found on from square")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pon,
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PieceType::King => "K",
                PieceType::Queen => "Q",
                PieceType::Rook => "R",
                PieceType::Bishop => "B",
                PieceType::Knight => "N",
                PieceType::Pon => "P",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Player {
    White,
    Black,
}

// NOTE(brick): If Vec is an Array, we can build a static board and let the
// array's position determine the coordinates
// OR we can just use a static array storing the 32 pieces and not worry about the board
// OR we can build a static array of references to pieces so we can avoid memory size
// calculation issue
impl Game {
    fn build_board(&mut self) -> &mut Self {
        let white_pon = Piece {
            piece_type: PieceType::Pon,
            position: Position { x: 3, y: 2 },
            side: Player::White,
        };

        let white_knight = Piece {
            piece_type: PieceType::Knight,
            position: Position { x: 2, y: 1 },
            side: Player::White,
        };

        self.board.push(white_pon);
        self.board.push(white_knight);

        self
    }
}

// TODO(brick): List all rules validation
// NOTE(brick): How do we solve complexe rule validation?
// TODO(brick): Here we browse the whole board to check for the piece. We might only want
// the piece type in order to validate the move
fn is_move_valid(board: Vec<Piece>, from: Position, to: Position) -> bool {
    let piece_idx = board
        .iter()
        .position(|p| p.position.x == from.x && p.position.y == from.y);

    match piece_idx {
        Some(idx) => {
            let piece = board[idx];

            match piece.piece_type {
                PieceType::King => {
                    (to.x == from.x + 1 && to.y == from.y)
                        || (to.x == from.x - 1 && to.y == from.y)
                        || (to.y == from.y + 1 && to.x == from.x)
                        || (to.y == from.y - 1 && to.x == from.x)
                        || (to.x == from.x + 1 && to.y == from.y + 1)
                        || (to.x == from.x - 1 && to.y == from.y + 1)
                        || (to.x == from.x + 1 && to.y == from.y - 1)
                        || (to.x == from.x - 1 && to.y == from.y - 1)
                }
                PieceType::Queen => false,
                PieceType::Rook => false,
                PieceType::Bishop => to.x - from.x == to.y - from.y,
                PieceType::Knight => {
                    (to.x == from.x + 1 && to.y == from.y + 2)
                        || (to.x == from.x + 1 && to.y == from.y - 2)
                        || (to.x == from.x - 1 && to.y == from.y + 2)
                        || (to.x == from.x - 1 && to.y == from.y - 2)
                        || (to.x == from.x + 2 && to.y == from.y + 1)
                        || (to.x == from.x + 2 && to.y == from.y - 1)
                        || (to.x == from.x - 2 && to.y == from.y + 1)
                        || (to.x == from.x - 2 && to.y == from.y - 1)
                }
                PieceType::Pon => {
                    (to.x == from.x && to.y == from.y + 1) || (from.x == 2 && to.y == from.y + 2)
                }
            }
        }
        None => false,
    }
}

// REFACTOR(brick): This can definitely be set up by populating the board vec! straight away
#[cfg(test)]
mod tests {
    
    #[test]
    fn moving_a_single_pon() {
        let mut blank_game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![],
        };

        let mut game = crate::Game::build_board(&mut blank_game);

        // NOTE(brick): c3 matches starting position from both Knight (b1)
        // and pon (c2) board.move_piece("pc3");
        game.move_piece(
            crate::Position { x: 3, y: 2 },
            crate::Position { x: 3, y: 3 },
        );

        // Pon to c4
        // game.move_piece("c4");

        assert_eq!(game.board[0].position, crate::Position { x: 3, y: 3 });
    }

    #[test]
    fn parsing_basic_move() {
        let result = crate::Position { x: 2, y: 2 };

        let position = match crate::string_to_position("b2") {
            Ok(position) => position,
            _ => panic!(String::from("Cannot parse this position")),
        };

        assert_eq!(position, result);
    }

    #[test]
    fn can_distinguish_piece_on_from_square() {
        let mut blank_game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![],
        };

        let mut game = crate::Game::build_board(&mut blank_game);

        game.move_piece(
            crate::Position { x: 2, y: 1 },
            crate::Position { x: 3, y: 3 },
        );

        let piece_type = game.moves[0].piece_type;

        let result = crate::PieceType::Knight;

        assert_eq!(piece_type, result);
    }

    #[test]
    #[should_panic]
    fn panics_in_case_from_square_is_empty() {
        let mut blank_game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![],
        };

        let mut game = crate::Game::build_board(&mut blank_game);

        game.move_piece(
            crate::Position { x: 4, y: 4 },
            crate::Position { x: 5, y: 4 },
        );
    }

    #[test]
    fn white_pon_capture_is_registered() {
        let mut game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![
                crate::Piece {
                    side: crate::Player::White,
                    position: crate::Position { x: 4, y: 4 },
                    piece_type: crate::PieceType::Pon,
                },
                crate::Piece {
                    side: crate::Player::Black,
                    position: crate::Position { x: 5, y: 5 },
                    piece_type: crate::PieceType::Pon,
                },
            ],
        };

        game.move_piece(
            crate::Position { x: 4, y: 4 },
            crate::Position { x: 5, y: 5 },
        );

        assert_eq!(game.moves[0].capture, true);
    }

    #[test]
    fn white_pon_capture_removes_black_pon() {
        let mut game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![
                crate::Piece {
                    side: crate::Player::White,
                    position: crate::Position { x: 4, y: 4 },
                    piece_type: crate::PieceType::Pon,
                },
                crate::Piece {
                    side: crate::Player::Black,
                    position: crate::Position { x: 5, y: 5 },
                    piece_type: crate::PieceType::Pon,
                },
            ],
        };

        game.move_piece(
            crate::Position { x: 4, y: 4 },
            crate::Position { x: 5, y: 5 },
        );

        assert_eq!(game.board.len(), 1);
        assert_eq!(game.board[0].side, crate::Player::White);
    }

    #[test]
    fn white_side_rocks_properly() {
        let mut game = crate::Game {
            player: crate::Player::White,
            moves: vec![],
            board: vec![
                crate::Piece {
                    side: crate::Player::White,
                    position: crate::Position {x:1, y: 4},
                    piece_type: crate::PieceType::King
                },
                crate::Piece {
                    side: crate::Player::White,
                    position: crate::Position {x:1, y: 1},
                    piece_type: crate::PieceType::Rook
                },                
            ]
        };

        game.move_piece(
            crate::Position { x: 1, y: 4},
            crate::Position { x: 1, y: 2},
        );

        assert_eq!(game.board[0].position.y, 2);
        assert_eq!(game.board[1].position.y, 3);
    }

    #[test]
    fn black_side_rocks_properly() {
        let mut game = crate::Game {
            player: crate::Player::Black,
            moves: vec![],
            board: vec![
                crate::Piece {
                    side: crate::Player::Black,
                    position: crate::Position {x:8, y: 4},
                    piece_type: crate::PieceType::King
                },
                crate::Piece {
                    side: crate::Player::Black,
                    position: crate::Position {x:8, y: 8},
                    piece_type: crate::PieceType::Rook
                },                
            ]
        };

        game.move_piece(
            crate::Position { x: 8, y: 4},
            crate::Position { x: 8, y: 6},
        );

        assert_eq!(game.board[0].position.y, 6);
        assert_eq!(game.board[1].position.y, 5);
    }

    // TODO(brick): The end game testing wise is outputing a valid PGN after a series of a few moves

    // fn white_pon_c3_registers_correctly() {
    //     let mut board = crate::build_board();

    //     board.move_piece("c2", "c3");

    //     assert_eq(format!("{}", game.moves), "Pc2 c3");
    // }

    // fn white_knight_c3_registers_correctly() {}

    // fn black_knight_can_capture() {}
}
