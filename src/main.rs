extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down
}

struct Game {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
    just_eaten: bool,
    rows: u32,
    cols: u32,
    square_width: u32,
}

struct Snake {
    gl: GlGraphics,
    snake_parts: LinkedList<SnakePiece>,
    direction: Direction,
    width: u32
}

pub struct Food {
    pos_x: u32,
    pos_y: u32,
}

#[derive(Clone)]
pub struct SnakePiece(i32, i32);

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        const BLACK: [f32; 4] = [0., 0., 0., 0.5];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(BLACK, gl);
        });

        self.snake.render(args);
        self.food.render(&mut self.gl, args, self.square_width)

    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        if !self.snake.update(self.just_eaten, self.cols, self.rows) {
            return false;
        }

        self.just_eaten = self.food.update(&self.snake);
        if self.just_eaten {
            use rand::Rng;
            use rand::thread_rng;
            // try my luck
            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0, self.cols);
                let new_y = r.gen_range(0, self.rows);
                if !self.snake.is_collide(new_x as i32, new_y as i32) {
                    self.food = Food { pos_x: new_x, pos_y: new_y };
                    break;
                }
            }
        }

        return true;
    }

    fn pressed(&mut self, button: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match button {
            &Button::Keyboard(Key::Up)
                if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down)
                if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left)
                if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right)
                if last_direction != Direction::Left => Direction::Right,
            _ => last_direction
        };
    }
}

impl Food {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {

        const RED: [f32; 4] = [1., 0., 0., 1.];

        let y = self.pos_y * width;
        let x = self.pos_x * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, snake: &Snake) -> bool {
        let front = snake.snake_parts.front().unwrap();

        if front.0 == self.pos_x as i32 && front.1 == self.pos_y as i32 {
            return true
        }
        else {
            return false
        }
    }
}   


impl Snake {
    fn render(&mut self, args: &RenderArgs) {

        const GREEN: [f32; 4] = [0., 0.75, 0., 1.];

        let squares: Vec<graphics::types::Rectangle> = self.snake_parts
            .iter()
            .map(|piece| SnakePiece(piece.0 * self.width as i32, piece.1 * self.width as i32))
            .map(|piece| graphics::rectangle::square(piece.0 as f64, piece.1 as f64, self.width as f64))
            .collect();

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(GREEN, square, transform, gl))
        });
    }

    fn is_collide(&self, x: i32, y: i32) -> bool {
        self.snake_parts.iter().any(|p| x == p.0 && y == p.1)
    }

    pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_head: SnakePiece = (*self.snake_parts.front().expect("Snake has no body")).clone();

        // collision into wall evolves into death
        // if (self.direction == Direction::Up && new_head.1 == 0) ||
        //    (self.direction == Direction::Left && new_head.0 == 0) ||
        //    (self.direction == Direction::Down && new_head.1 == rows - 1) ||
        //    (self.direction == Direction::Right && new_head.0 == cols - 1) {
        //     return false;
        // }

        if new_head.0 == 0 && self.direction == Direction::Left {
            new_head.0 = cols as i32; // teleport to the right of the map
        } else if self.direction == Direction::Right && new_head.0 == (cols - 1) as i32 {
            new_head.0 = -1; // teleport to the left of the map
        }

        if new_head.1 == 0 && self.direction == Direction::Up {
            new_head.1 = rows as i32; // teleport to the bottom of the map
        } else if new_head.1 == (rows - 1) as i32 && self.direction == Direction::Down {
            new_head.1 = -1; // teleport to the top of the map
        }

        match self.direction {
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }

        if !just_eaten {
            self.snake_parts.pop_back();
        }

        if self.is_collide(new_head.0, new_head.1) {
            return false;
        }

        self.snake_parts.push_front(new_head);
        true
    }

    
}


fn main() {
    let opengl = OpenGL::V3_2;

    const COLS: u32 = 30;
    const ROWS: u32 = 20;
    const SQUARE_WIDTH: u32 = 20;

    const WIDTH: u32 = COLS * SQUARE_WIDTH;
    const HEIGHT: u32 = ROWS * SQUARE_WIDTH;

    let mut window: GlutinWindow = WindowSettings::new(
        "Snekk Game",
        [WIDTH, HEIGHT]
    )
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();
    
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLS,
        square_width: SQUARE_WIDTH,
        just_eaten: false,
        food: Food {pos_x: 10, pos_y: 10},
        snake: Snake {
            gl: GlGraphics::new(opengl), 
            snake_parts: LinkedList::from_iter((vec![SnakePiece(0,0)]).into_iter()),
            direction: Direction::Down,
            width: SQUARE_WIDTH,
        }
    };

    let mut events = Events::new(EventSettings::new()).ups(15 as u64);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
           if !game.update(&u) {
               break;
           }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
}
