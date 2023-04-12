mod snake;

use crate::snake::point::*;
use crate::snake::*;
use std::io::{stdin, stdout};

const DISPLAY_SIZE: usize = 15;

fn main() {
	let mut game = SnakeGame {
		positions: vec![Point::random_point(DISPLAY_SIZE as u8)],
		apple: Point::random_point(DISPLAY_SIZE as u8),
		stdin: stdin(),
		out: stdout()
	};

	game.start_game::<DISPLAY_SIZE>();
}