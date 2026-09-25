use std::slice::from_raw_parts;

use ggez::{input::mouse, *};
use chess::*;
use ggez::input::mouse::MouseButton;

struct State {
    board: Vec<Vec<bool>>,
    img_board: graphics::Image,
    img_chess_pieces: graphics::Image,
    from_square: Option<usize>,
    to_square: Option<usize>,
    turn: i32,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let mut boardd = vec![vec![false; 64]; 14];
        init(&mut boardd);
        let board= boardd;
        let img_board = match graphics::Image::from_path(ctx, "/img_board.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_chess_pieces = match graphics::Image::from_path(ctx, "/img_chess_pieces.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let from_square = None;
        let to_square = None;
        let turn = 0; // white starts
        Ok(State {board, img_board, img_chess_pieces, from_square, to_square, turn})
    }

    fn draw_piece(&self, canvas: &mut ggez::graphics::Canvas, color: f32, piece: f32, x: f32, y: f32) {
        canvas.draw (
            &self.img_chess_pieces,
            graphics::DrawParam::new()
                .src(graphics::Rect::new(
                    // x is left to right which piece
                    // y is white or black
                    piece / 6.0,
                    color / 2.0,
                    1.0 / 6.0,
                    1.0 / 2.0,
                ))
                .dest([x,y])
                .scale([0.35,0.35])
        );
    }
}



impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            // mouse position
            let mouse_posisiton = ctx.mouse.position();
            let (s_width, s_height) = ctx.gfx.drawable_size(); // returns drawable window size
            let imgb_width = self.img_board.width() as f32;
            let imgb_height = self.img_board.height() as f32;
            let board_x = (s_width - imgb_width) / 2.0;
            let board_y = (s_height - imgb_height) / 2.0;
            // col and row of the mouse click
            let col = ((mouse_posisiton.x - board_x)/ (imgb_width/8.0)).floor();
            let row = ((mouse_posisiton.y - board_y)/ (imgb_width/8.0)).floor();

            if mouse_posisiton.x < board_x || mouse_posisiton.y < board_y || mouse_posisiton.x > board_x+imgb_width || mouse_posisiton.y > board_y+imgb_width {
                return Ok(())
            }
            else if let None = self.from_square {
                self.from_square = Some((row*8.0+col) as usize);
            }
            else {
                self.to_square = Some((row*8.0+col) as usize);
            }

            if self.turn == 0 && piece_at(&self.board, self.from_square.unwrap())<6 {
                if let Some(_) = self.from_square {
                    if let Some(_) = self.to_square {
                        // (start_row * 8 + start_column) * 64 + end_row * 8 + end_column
                        let uci = move_to_uci(((self.from_square).unwrap() * 64 + (self.to_square).unwrap()) as i32);
                        let board1 = self.board.clone();
                        // will make move if legal!!
                        let (result, board) = make_move(board1, &uci);
                        self.board = board;
                        // set the chosen squares to none again after making move
                        self.from_square = None;
                        self.to_square = None;
                        if result == true {
                            self.turn = 1;
                        }
                    }
                }
            }
            else if self.turn == 1 && piece_at(&self.board, self.from_square.unwrap())>5 {
                if let Some(_) = self.from_square {
                    if let Some(_) = self.to_square {
                        // (start_row * 8 + start_column) * 64 + end_row * 8 + end_column
                        let uci = move_to_uci(((self.from_square).unwrap() * 64 + (self.to_square).unwrap()) as i32);
                        let board1 = self.board.clone();
                        // will make move if legal!!
                        let (result, board) = make_move(board1, &uci);
                        self.board = board;
                        // set the chosen squares to none again after making move
                        self.from_square = None;
                        self.to_square = None;
                        if result == true {
                            self.turn = 0;
                        }
                    }
                }
            }
            

            println!("{:?} {:?}", self.from_square, self.to_square);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (s_width, s_height) = ctx.gfx.drawable_size(); // returns drawable window size
        let mut canvas = graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);

        let imgb_width = self.img_board.width() as f32;
        let imgb_height = self.img_board.height() as f32;
        let board_x = (s_width - imgb_width)/2.0;
        let board_y = (s_height - imgb_height)/2.0;
        let squ = imgb_width/8.0;

        canvas.draw (
            &self.img_board,
            graphics::DrawParam::new()
                .dest([(s_width-imgb_width)/2.0,(s_height-imgb_height)/2.0])
                .scale([1.0,1.0])
        );
        
        for i in 0..64 {
            let p = piece_at(&self.board, i as usize) as f32;
            /*  0  = white pawn
                1  = white rook
                2  = white knight
                3  = white bishop
                4  = white queen
                5  = white king
                6  = black pawn
                7  = black rook
                8  = black knight
                9  = black bishop
                10 = black queen
                11 = black king   */
            let color = (p/6.0).floor();
            let piece = 5.0-p%6.0;
            let col = (i%8) as f32;
            let row = (i/8) as f32;
            let x = board_x + col * squ;
            let y = board_y + row * squ;
            self.draw_piece(&mut canvas, color, piece, x, y);
        }
        
        canvas.finish(ctx)?;
        Ok(())
    }

    
}


pub fn main() {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("hello_ggez", "awesome_person")
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx).expect("fail");
    event::run(ctx, event_loop, state);

    // legal moves: 0-4095 (start_row * 8 + start_column) * 64 + end_row * 8 + end_column
    // let moves = find_legal_moves(&mut board, 0);
    // let uci = move_to_uci(3981)
    // let (result, board) = make_move(board, "c1f4");
    // The result will tell you if the move was legal or not
    /*
        0  = white pawn
        1  = white rook
        2  = white knight
        3  = white bishop
        4  = white queen
        5  = white king
        6  = black pawn
        7  = black rook
        8  = black knight
        9  = black bishop
        10 = black queen
        11 = black king 
    */
}