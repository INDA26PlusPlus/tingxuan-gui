use ggez::{graphics, *};
use chess::*;
use ggez::input::mouse::MouseButton;
use ggez::audio::SoundSource;

struct State {
    board: Vec<Vec<bool>>,
    // images
    img_bground: graphics::Image,
    img_board: graphics::Image,
    img_chess_pieces: graphics::Image,
    img_jumino: graphics::Image,
    // music and sound
    bgm_danger: ggez::audio::Source,
    sfx_move: ggez::audio::Source,
    sfx_select: ggez::audio::Source,
    playlist: [ggez::audio::Source; 4],
    playing: usize,
    // keep track of active squares
    from_square: Option<usize>,
    to_square: Option<usize>,
    turn: i32,
    white_check: bool,
    black_check: bool
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let mut boardd = vec![vec![false; 64]; 14];
        // initialize board
        init(&mut boardd);
        let board= boardd;

        // region: load pictures, if error return error else return image
        let img_bground =  match graphics::Image::from_path(ctx, "/img_bground.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_board = match graphics::Image::from_path(ctx, "/img_board.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_chess_pieces = match graphics::Image::from_path(ctx, "/img_chess_pieces.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_jumino = match graphics::Image::from_path(ctx, "/img_jumino.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        // endregion

        // region: load sound and music
        let bgm1 = match ggez::audio::Source::new(ctx, "/bgm1.mp3") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let bgm2 = match ggez::audio::Source::new(ctx, "/bgm2.mp3") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let bgm3 = match ggez::audio::Source::new(ctx, "/bgm3.mp3") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let bgm4 = match ggez::audio::Source::new(ctx, "/bgm4.mp3") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let bgm_danger = match ggez::audio::Source::new(ctx, "/bgm_danger.mp3") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let sfx_move = match ggez::audio::Source::new(ctx, "/sfx_move.wav") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let sfx_select = match ggez::audio::Source::new(ctx, "/sfx_select.wav") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        // endregion
        let playlist = [bgm4,bgm1,bgm2,bgm3];
        let playing = 0;
        // start with no squares selected
        let from_square = None;
        let to_square = None;
        let turn = 0; // white starts
        let white_check = false;
        let black_check = false;

        Ok(State {board, img_bground, img_board, img_chess_pieces, img_jumino, bgm_danger, sfx_move, sfx_select,from_square, playlist, playing, to_square, turn,white_check,black_check})
    }

    // function to draw chess piece on square
    fn draw_piece(&self, canvas: &mut ggez::graphics::Canvas, color: f32, piece: f32, x: f32, y: f32, squ: f32) {
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
                .scale([squ/333.0,squ/333.0])
        );
    }

    // funtion to highlight a square if selected
    fn square_highlight(&self, canvas: &mut ggez::graphics::Canvas, ctx: &mut Context, x:f32, y:f32, squ: f32, r: f32, g:f32, b:f32, a:f32) {
        // highlight
        let highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0,0.0,squ,squ),
            graphics::Color::new(r, g, b, a)
        );
        if let Result::Ok(_) = highlight {
            canvas.draw(
                &highlight.unwrap(),
                graphics::DrawParam::new()
                    .dest([x,y])
            );
        }

    }

    fn draw_jumino(&self, canvas: &mut ggez::graphics::Canvas, x:f32, y:f32) {
        canvas.draw (
            &self.img_jumino,
            graphics::DrawParam::new()
                .dest([x,y])
                .scale([0.15,0.15])
        );
    }


}

impl ggez::event::EventHandler for State {
    
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.bgm_danger.playing() == false {
            if self.playlist[self.playing].stopped() {
                self.playing = (self.playing+1)%4;
                self.playlist[self.playing].play();
            }
            else if self.playlist[self.playing].paused() {
                self.playlist[self.playing].resume();
            }
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

        // draw background, bottom layer
        canvas.draw (
            &self.img_bground,
            graphics::DrawParam::new()
                .dest([0.0,0.0])
                .scale([1.6,1.6])
        );
        // draw image of chess board
        canvas.draw (
            &self.img_board,
            graphics::DrawParam::new()
                .dest([(s_width-imgb_width)/2.0,(s_height-imgb_height)/2.0])
                .scale([1.0,1.0])
        );
        
        // highlight the chosen square
        if self.from_square != None {
            let pos = self.from_square.unwrap();
            let col = (pos%8) as f32;
            let row = (pos/8) as f32;
            self.square_highlight(&mut canvas, ctx, board_x + col * squ, board_y + row * squ, squ,0.0,0.0,100.0,0.8);
        }

        // if in check, highlight that square
        if self.white_check {
            for i in 0..64 {
                let p = piece_at(&self.board, i as usize) as f32;
                let col = (i%8) as f32;
                let row = (i/8) as f32;
                if p == 5.0 {
                    self.square_highlight(&mut canvas, ctx, board_x + col * squ, board_y + row * squ, squ,100.0,0.0,10.0,0.6);
                }
            }
        }
        else if self.black_check {
            for i in 0..64 {
                let p = piece_at(&self.board, i as usize) as f32;
                let col = (i%8) as f32;
                let row = (i/8) as f32;
                if p == 11.0 {
                    self.square_highlight(&mut canvas, ctx, board_x + col * squ, board_y + row * squ, squ,100.0,0.0,10.0,0.6);
                }
            }
        }
        else {
            self.bgm_danger.stop()
        }

        // draw the chess pieces as in board
        for i in 0..64 {
            let p = piece_at(&self.board, i as usize) as f32;
            let color = (p/6.0).floor();
            let piece = 5.0-p%6.0;
            let col = (i%8) as f32;
            let row = (i/8) as f32;
            let x = board_x + col * squ;
            let y = board_y + row * squ;
            self.draw_piece(&mut canvas, color, piece, x, y, squ);
        }
        
        // draw the legal moves at top layer
        if self.from_square != None {
            // for each move in the legal moves
            for mv in find_legal_moves(&mut self.board, self.turn+1) { // 0->1, 1->2
                let uci = move_to_uci(mv);
                let convert: [char; 8] = ['a','b','c','d','e','f','g','h'];
                let col0 = self.from_square.unwrap()%8;
                let row0 = self.from_square.unwrap()/8;

                // iterate to find index of letter in convert
                let col1 = convert.iter().position(|&c| c == (uci[2..3]).parse().unwrap()).unwrap();
                let row1: f32 = (&uci[3..4]).parse().unwrap();

                // if this legal move is for the piece we selected
                if &uci[0..2] == format!("{}{}",convert[col0],8-row0) {
                    let x = board_x+29.0 + (col1 as f32) * squ;
                    let y = board_y+33.0 + (8.0 - row1) * squ;
                    self.draw_jumino(&mut canvas, x, y);
                }
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) -> Result<(), GameError> {
        // if somewhere clicked on
        if _button == MouseButton::Left {
            // mouse position
            let mouse_posisiton = _ctx.mouse.position();
            let (s_width, s_height) = _ctx.gfx.drawable_size(); // returns drawable window size
            let imgb_width = self.img_board.width() as f32;
            let imgb_height = self.img_board.height() as f32;
            let board_x = (s_width - imgb_width) / 2.0;
            let board_y = (s_height - imgb_height) / 2.0;
            // outside the board then nothing happens
            if mouse_posisiton.x < board_x || mouse_posisiton.y < board_y || mouse_posisiton.x > board_x+imgb_width || mouse_posisiton.y > board_y+imgb_width {
                return Ok(());
            }
           
            // col and row of the mouse click
            let col = ((mouse_posisiton.x - board_x)/ (imgb_width/8.0)).floor();
            let row = ((mouse_posisiton.y - board_y)/ (imgb_width/8.0)).floor();
            let piece = piece_at(&self.board, (row*8.0+col) as usize);

            // if no selected square, or selected square before was my piece
            if self.from_square == None || (piece!=-1 && (piece/6)==self.turn){
                // check so its not opponents piece
                if piece!=-1 && (piece/6)==self.turn {
                    self.from_square = Some((row*8.0+col) as usize);
                    self.sfx_select.play();
                }
            }    
            else { // else we have selected to_square
                self.to_square = Some((row*8.0+col) as usize);   
            }

            // has selected from_square and to_square, from_square is current turn piece
            if self.from_square != None && piece_at(&self.board, self.from_square.unwrap())/6 == self.turn && self.to_square!=None{
                // (start_row * 8 + start_column) * 64 + end_row * 8 + end_column
                let uci = move_to_uci(((self.from_square).unwrap() * 64 + (self.to_square).unwrap()) as i32);
                let (result, board) = make_move(self.board.clone(), &uci); // make move if legal
                self.board = board;
                // reset selected squares
                self.from_square = None;
                self.to_square = None;
                // if move successful
                if result == true {
                    self.turn = (self.turn+1)%2; // next player turn
                    self.black_check = false; // reset check values
                    self.white_check = false;
                    for i in 0..64 { 
                        let col = i%8;
                        let row = i/8;
                        let convert: [char; 8] = ['a','b','c','d','e','f','g','h'];
                        // if black king check
                        if piece_at(&self.board, i as usize) == 11 {
                            for mv in find_legal_moves(&mut self.board, 1) { // 1 skips black pieces
                                let uci = move_to_uci(mv);
                                let col1 = convert.iter().position(|&c| c == (uci[2..3]).parse().unwrap()).unwrap();
                                let row1: usize = (&uci[3..4]).parse().unwrap();
                                if col == col1 && row == 8-row1 {
                                    self.black_check = true;
                                    self.playlist[self.playing].pause();
                                    self.bgm_danger.set_repeat(true);
                                    self.bgm_danger.play();
                                    break;
                                }
                            }
                        }
                        // if white king check
                        else if piece_at(&self.board, i as usize) == 5 {
                            for mv in find_legal_moves(&mut self.board, 2) { // 2 skips white pieces
                                let uci = move_to_uci(mv);
                                let col1 = convert.iter().position(|&c| c == (uci[2..3]).parse().unwrap()).unwrap();
                                let row1: usize = (&uci[3..4]).parse().unwrap();
                                if col == col1 && row == 8-row1 {
                                    self.white_check = true;
                                    self.playlist[self.playing].pause();
                                    self.bgm_danger.set_repeat(true);
                                    self.bgm_danger.play();
                                    break;
                                }
                            }
                        }
                    }
                    self.sfx_move.play(); // play sound effect lastly
                }
            }
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("chessdew valley", "tingxuan")
        .default_conf(c)
        .build()?; // ? means if error, return right away

    let state = State::new(&mut ctx)?;
    event::run(ctx, event_loop, state) // return gameresult from this
}

// TODO: stalemate and checkmate end game

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