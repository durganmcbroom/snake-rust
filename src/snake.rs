    use err::SnakeErrorRecoverable;
    use std::marker::PhantomData;

    pub mod err {
        use std::fmt;

        pub trait SnakeError: fmt::Debug {
            fn get_error(&self) -> String;
        }

        pub trait SnakeErrorRecoverable<T> {
            fn recover_or_panic(self: Self) -> T;
        }

        impl<T> SnakeErrorRecoverable<T> for Result<T, Box<dyn SnakeError>> {
            fn recover_or_panic(self: Self) -> T {
                return self.unwrap();
            }
        }

        pub enum BasicSnakeError {
            UnknownKey(char),
            EmptyLine,
            CantReadLine(std::io::Error),
        }

        impl fmt::Debug for BasicSnakeError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.get_error())
            }
        }

        impl SnakeError for BasicSnakeError {
            fn get_error(&self) -> String {
                match self {
                    // I like, but weird lol
                    BasicSnakeError::UnknownKey(key_pressed) => {
                        format!("Unknown key pressed '{0}'.", key_pressed)
                    }
                    BasicSnakeError::EmptyLine => {
                        format!("Failed to parse input from line because its empty!")
                    }
                    BasicSnakeError::CantReadLine(err) => {
                        format!("{0}", err)
                    }
                }
            }
        }
    }

    pub mod render {
        use std::io;
        use std::marker::PhantomData;

        pub trait DisplayRenderer {
            fn render(on: &u8) -> String;
        }

        pub struct SnakeRenderer {}

        impl DisplayRenderer for SnakeRenderer {
            fn render(on: &u8) -> String {
                return match *on {
                    0 => String::from("[ ]"),
                    1 => String::from("[@]"),
                    2 => String::from("[a]"),
                    3 => String::from("[%]"),
                    _ => String::from("[ ]"),
                };
            }
        }

        pub struct MyDisplay<T: DisplayRenderer + Sized, const S: usize> {
            pub content: [[u8; S]; S],
            pub _marker: PhantomData<T>, // renderer: T,
        }

        impl<T: DisplayRenderer, const S: usize> MyDisplay<T, S> {
            pub fn draw(self: &MyDisplay<T, S>, writer: &mut impl io::Write) {
                for a in self.content.iter() {
                    let mut row = String::new();
                    for b in a.iter() {
                        row.push_str(&T::render(b))
                    }
                    write!(writer, "{}\n", row).unwrap();
                }
            }
        }
    }

    pub mod point {
        use rand::Rng;

        #[derive(Copy, Clone, Debug)]
        pub struct Point {
            pub x: i16,
            pub y: i16,
        }

        impl Point {
            pub fn random_point(largest: u8) -> Point {
                let mut rng = rand::thread_rng();
                Point {
                    x: rng.gen_range(0..largest) as i16,
                    y: rng.gen_range(0..largest) as i16,
                }
            }

            pub fn out_of_bounds(&self, bound: u8) -> bool {
                !(self.x >= 0 && self.x < bound as i16 && self.y >= 0 && self.y < bound as i16)
            }
        }

        pub fn shift_positions(positions: &mut Vec<Point>, transformer: impl Fn(&mut Point)) {
            for i in (1..positions.len()).rev() {
                let previous = positions[i - 1];
                let pos = &mut positions[i];

                pos.x = previous.x;
                pos.y = previous.y;
            }

            transformer(&mut positions[0]);
        }
    }

    pub mod input {
        use super::err::*;
        use std::io::Stdin;

        pub enum InputKey {
            W,
            S,
            A,
            D,
            Quit,
        }

        impl InputKey {
            pub fn read_line(stdin: &Stdin) -> Result<InputKey, Box<dyn SnakeError>> {
                let mut line = String::new();

                let res = stdin
                    .read_line(&mut line)
                    .map_err(|err| Box::new(BasicSnakeError::CantReadLine(err)));
                if let Result::Err(e) = res {
                    return Result::Err(e);
                }

                let c = line
                    .chars()
                    .nth(0)
                    .ok_or_else(|| Box::new(BasicSnakeError::EmptyLine) as Box<dyn SnakeError>)?;

                return InputKey::from(c);
            }

            pub fn from(c: char) -> Result<InputKey, Box<dyn SnakeError>> {
                return match c {
                    'w' => Result::Ok(InputKey::W),
                    's' => Result::Ok(InputKey::S),
                    'a' => Result::Ok(InputKey::A),
                    'd' => Result::Ok(InputKey::D),
                    ' ' => Result::Ok(InputKey::Quit),
                    _ => Result::Err(Box::new(BasicSnakeError::UnknownKey(c))),
                };
            }
        }
    }

    pub struct SnakeGame<T>
    where
        T: std::io::Write,
    {
        pub positions: Vec<point::Point>,
        pub apple: point::Point,
        pub stdin: std::io::Stdin,
        pub out: T,
    }

    impl<T: std::io::Write> SnakeGame<T> {
       pub fn start_game<const S: usize>(&mut self) {
            let display = self.create_snake_frame::<render::SnakeRenderer, S>();
            display.draw(&mut self.out);

            loop {
                if !self.tick::<S>() {
                    break;
                }
            }
        }

        fn tick<const S: usize>(&mut self) -> bool {
            let key = input::InputKey::read_line(&self.stdin).recover_or_panic();

            let transformer = match key {
                input::InputKey::W => |it: &mut point::Point| {
                    it.y -= 1;
                },
                input::InputKey::S => |it: &mut point::Point| {
                    it.y += 1;
                },
                input::InputKey::D => |it: &mut point::Point| {
                    it.x += 1;
                },
                input::InputKey::A => |it: &mut point::Point| {
                    it.x -= 1;
                },
                input::InputKey::Quit => return false,
            };

            return self.update_frame::<S>(transformer);
        }

        fn update_frame<const S: usize>(
            &mut self,
            // positions: &mut Vec<point::Point>,
            // apple: &mut point::Point,
            transformer: impl Fn(&mut point::Point),
        ) -> bool {
            point::shift_positions(&mut self.positions, transformer);

            let first = self.positions[0];

            if first.out_of_bounds(S as u8) {
                return false;
            } else if self
                .positions
                .iter()
                .enumerate()
                .any(|(i, it)| first.x == it.x && first.y == it.y && i != 0)
            {
                return false;
            }

            if self
                .positions
                .iter()
                .any(|it| it.x == self.apple.x && it.y == self.apple.y)
            {
                let new_apple = point::Point::random_point(S as u8);

                self.apple.x = new_apple.x;
                self.apple.y = new_apple.y;
                self.positions
                    .push(self.positions[self.positions.len() - 1].clone())
            }

            let display = self.create_snake_frame::<render::SnakeRenderer, S>();

            display.draw(&mut self.out);

            true
        }

        fn create_snake_frame<R: render::DisplayRenderer + Sized, const S: usize>(
            &mut self,
            // positions: &Vec<point::Point>,
            // apple: &point::Point,
        ) -> render::MyDisplay<R, S> {
            let mut screen_content: [[u8; S]; S] = [[0; S]; S];

            for y in 0..S {
                for x in 0..S {
                    screen_content[y][x] = 0;
                }
            }

            screen_content[self.apple.y as usize][self.apple.x as usize] = 2;
            screen_content[self.positions[0].y as usize][self.positions[0].x as usize] = 3;

            for i in 1..self.positions.len() {
                let pos = self.positions[i];
                screen_content[pos.y as usize][pos.x as usize] = 1;
            }

            render::MyDisplay {
                content: screen_content,
                _marker: PhantomData,
            }
        }
    }
