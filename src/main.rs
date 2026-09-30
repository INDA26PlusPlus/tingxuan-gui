// region: import
use ggez::{graphics, *};
use chess::*;
use ggez::input::mouse::MouseButton;
use ggez::audio::SoundSource;
use std::fmt::Error;
use std::net::{TcpListener, TcpStream};
use std::env;
use std::io::{ErrorKind, prelude::*};
// endregion

struct State {
    board: Vec<Vec<bool>>,
    my_color: i32,
    stream: TcpStream,
    buffer: String,
    new_board: Vec<Vec<bool>>,
    // images
    img_bground: graphics::Image,
    img_board: graphics::Image,
    img_chess_pieces: graphics::Image,
    img_jumino: graphics::Image,
    img_win_black: graphics::Image,
    img_win_white: graphics::Image,
    img_draw: graphics::Image,
    img_letter: graphics::Image,
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
    black_check: bool,
    white_stalemate: bool,
    black_stalemate: bool
}

impl State {
    fn new(ctx: &mut Context, instream: TcpStream, color: i32) -> GameResult<State> {
        let mut boardd = vec![vec![false; 64]; 14];
        // initialize board
        init(&mut boardd);
        let board= boardd;
        let my_color = color; 
        let stream = instream;
        let buffer= String::new();
        let new_board = board.clone(); // clone as i cannot borrow or move it

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
        let img_win_black = match graphics::Image::from_path(ctx, "/img_win_black.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_win_white = match graphics::Image::from_path(ctx, "/img_win_white.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_draw = match graphics::Image::from_path(ctx, "/img_draw.png") {
            Ok(val) => val,
            Err(e) => {println!("{e}"); return Err(e);},
        };
        let img_letter = match graphics::Image::from_path(ctx, "/img_letter.png") {
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
        let white_stalemate = false;
        let black_stalemate = false;

        Ok(State {board, my_color, stream, buffer, new_board, img_bground, img_board, img_chess_pieces, img_jumino, img_win_black, img_win_white, img_draw, img_letter, bgm_danger, sfx_move, sfx_select,from_square, playlist, playing, to_square, turn, white_check, black_check, white_stalemate, black_stalemate})
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

        // try to read response
        let mut data = [0u8; 128]; // all set to 0
        match self.stream.read(&mut data) {
            Ok(n) => {
                for i in 0..n {
                    if data[i] == b'\n' { // if the message is full
                        if self.buffer == "OK" || self.buffer == "CHECKMATE" || self.buffer == "STALEMATE" {
                            self.board = self.new_board.clone();
                            self.turn = (self.turn+1)%2; // next player turn
                            self.black_check = false; // reset check values
                            self.white_check = false;
                            let convert: [char; 8] = ['a','b','c','d','e','f','g','h'];

                            for i in 0..64 { 
                                let col = i%8;
                                let row = i/8;

                                // if black king check
                                if piece_at(&self.board, i as usize) == 11 {
                                    for mv in find_legal_moves(&mut self.board, 1) {
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
                                    for mv in find_legal_moves(&mut self.board, 2) {
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

                            self.black_stalemate = true;
                            self.white_stalemate = true;
                            for mv in find_legal_moves(&mut self.board, 0) {
                                if !self.black_stalemate && !self.white_stalemate {break}
                                let uci = move_to_uci(mv);
                                let col = convert.iter().position(|&c| c == (uci[0..1]).parse().unwrap()).unwrap();
                                let row: usize = (&uci[1..2]).parse().unwrap();
                                if piece_at(&self.board, (8-row)*8+col) < 6 && piece_at(&self.board, (8-row)*8+col) != -1 {self.white_stalemate = false;} // if white piece can move
                                else if piece_at(&self.board, (8-row)*8+col) > 5 {self.black_stalemate = false;} // if black piece can move
                            }
                            self.sfx_move.play(); // play sound effect lastly
                        }

                        else if self.buffer == "REJECT" {println!("move rejected");}

                        else {
                            let mut uci = self.buffer[0..4].to_lowercase(); // lowercase for the make_move
                            if &self.buffer[4..5] != "-" {uci += &self.buffer[4..5].to_lowercase()}; // if promotion
                            let (result, board) = make_move(self.board.clone(), &uci); 
                            // if the move is legal, send OK
                            if result {
                                self.black_check = false; // reset check values
                                self.white_check = false;
                                let convert: [char; 8] = ['a','b','c','d','e','f','g','h'];
                                for i in 0..64 { 
                                    let col = i%8;
                                    let row = i/8;
                                    // if black king check
                                    if piece_at(&self.board, i as usize) == 11 {
                                        for mv in find_legal_moves(&mut self.board, 1) {
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
                                        for mv in find_legal_moves(&mut self.board, 2) {
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
                                self.black_stalemate = true;
                                self.white_stalemate = true;
                                for mv in find_legal_moves(&mut self.board, 0) {
                                    if !self.black_stalemate && !self.white_stalemate {break}
                                    let uci = move_to_uci(mv);
                                    let col = convert.iter().position(|&c| c == (uci[0..1]).parse().unwrap()).unwrap();
                                    let row: usize = (&uci[1..2]).parse().unwrap();
                                    if piece_at(&self.board, (8-row)*8+col) < 6 && piece_at(&self.board, (8-row)*8+col) != -1 {self.white_stalemate = false;} // if white piece can move
                                    else if piece_at(&self.board, (8-row)*8+col) > 5 {self.black_stalemate = false;} // if black piece can move
                                }

                                if self.turn == 0 && self.white_stalemate {
                                    if self.white_check {self.stream.write(b"CHECKMATE\n")?;}
                                    else {self.stream.write(b"STALEMATE\n")?;}
                                }
                                else if self.turn == 1 && self.black_stalemate {
                                    if self.black_check {self.stream.write(b"CHECKMATE\n")?;}
                                    else {self.stream.write(b"STALEMATE\n")?;}
                                }
                                else {
                                    self.stream.write(b"OK\n")?;
                                }
                                self.board=board; 
                                self.sfx_move.play();
                                self.turn = (self.turn+1)%2;
                            }
                            // else illegal, REJECT
                            else {self.stream.write(b"REJECT\n")?;}
                        }

                        self.buffer=String::new(); // message is full, reset buffer
                    }
                    else {self.buffer.push(data[i] as char)}}
                },  
            Err(e) if e.kind() == ErrorKind::WouldBlock => {},
            Err(e) => {return Err(e.into())}
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
        // draw letter to the left
        // let imgl_width = self.img_letter.width() as f32;
        // let imgl_height = self.img_letter.height() as f32;
        // canvas.draw (
            // &self.img_letter,
            // graphics::DrawParam::new()
                // .dest([(s_width-imgl_width)/17.0,(s_height-imgl_height)/2.0])
                // .scale([1.0,1.0])
        // );

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
        
        // draw legal moves 
        if self.from_square != None {
            // for each move in the legal moves
            for mv in find_legal_moves(&mut self.board, 0) { // 0->1, 1->2
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

        // ending screens at top layer
        // stalemate + check -> opponent win
        if self.white_check && self.white_stalemate {
            let winb_width = self.img_win_black.width() as f32;
            let winb_height = self.img_win_black.height() as f32;
            canvas.draw (
                &self.img_win_black,
                graphics::DrawParam::new()
                    .dest([(s_width-winb_width)/2.0,(s_height-winb_height)/2.0])
                    .scale([1.0,1.0])
            );
        }
        else if self.black_check && self.black_stalemate {
            let winw_width = self.img_win_white.width() as f32;
            let winw_height = self.img_win_white.height() as f32;
            canvas.draw (
                &self.img_win_white,
                graphics::DrawParam::new()
                    .dest([(s_width-winw_width)/2.0,(s_height-winw_height)/2.0])
                    .scale([1.0,1.0])
            );
        }
        // stalemate -> draw
        else if self.black_stalemate || self.white_stalemate {
            let draw_width = self.img_draw.width() as f32;
            let draw_height = self.img_draw.height() as f32;
            canvas.draw (
                &self.img_draw,
                graphics::DrawParam::new()
                    .dest([(s_width-draw_width)/2.0,(s_height-draw_height)/2.0])
                    .scale([1.0,1.0])
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) -> Result<(), GameError> {
        // if somewhere clicked on
        if _button == MouseButton::Left && self.my_color == self.turn {
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
                // is move legal?
                let (result, board) = make_move(self.board.clone(), &uci); 
                // reset selected squares
                self.from_square = None;
                self.to_square = None;

                // if this is legal move for me, i try to send to other player
                if result {
                    self.new_board = board;
                    // send move to stream A1B2P[char; 64] (string och sedan .as_bytes())
                    let mut send_board = String::new();
                    let pieces = ["p","r","n","b","q","k"]; // easily convert number to piece
                    for i in 0..64 {
                        let p = piece_at(&self.new_board, i as usize) as i32;
                        if p == -1 {send_board+=" "}
                        else if p/6 == 0 {send_board+=&(pieces[p as usize]).to_uppercase();}
                        else {send_board+=&(pieces[(p%6) as usize])};
                    }
                    let send = uci.to_uppercase()+"Q"+&send_board+"\n";
                    self.stream.write_all(send.as_bytes())?; // retry to write until all                
                }
            }
        }
        Ok(())
    }
}

fn connect_game(ip: &str) -> Result<(TcpStream, i32), Error> {
    let mut stream = TcpStream::connect(ip).expect("connecting error");
    let mut buffer: [u8; 2] = [0; 2];
    stream.read(&mut buffer).expect("reading error");
    stream.set_nonblocking(true).expect("noneblocking error");
    println!("{:?}", buffer);
    // color of the client
    if buffer[0] == b'W' {return Ok((stream, 0))}
    else if buffer[0] == b'B' {return Ok((stream, 1))}
    Err(Error)
}

fn host_game(color: &str) -> Result<(TcpStream, i32), Error> {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6767").expect("binding error"); 
    let (mut stream, address) = listener.accept().expect("accept listener error");
    stream.set_nonblocking(true).expect("noneblocking error");
    if color == "--W"{
        stream.write(b"W\n").expect("writing error"); 
        let my_color=1;
        return Ok((stream, my_color))
    }
    else if color == "--B" {
        stream.write(b"B\n").expect("writing error");
        let my_color =0;
        return Ok((stream, my_color))
    }
    Err(Error)
}

// TcpStream  .write(bytes/&[u8]) or writeall?
//            .read(bytes)
// how to make string to &[u8] in rust?

pub fn main() -> GameResult {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("chessdew valley", "tingxuan")
        .default_conf(c)
        .window_mode(conf::WindowMode::default().resizable(true))
        .build()?; // ? means if error, return right away

    let args: Vec<String> = env::args().collect();
    if args[1] == "--host" {
        let (stream, my_color) = host_game(&args[2]).expect("host game error");
        let state = State::new(&mut ctx, stream, my_color)?;
        return event::run(ctx, event_loop, state)
    }
    
    if args[1] == "--connect" { 
        let (stream, my_color) = connect_game(&args[2]).expect("error");
        let state = State::new(&mut ctx, stream, my_color)?;
        return event::run(ctx, event_loop, state)
    }
    println!("{:?}",args);

    // let state = State::new(&mut ctx)?;
    // event::run(ctx, event_loop, state) // return gameresult from this
    Ok(())
} 

// when hosting: cargo run -- --host --[color of opponent]
// when connecting: cargo run -- --connect