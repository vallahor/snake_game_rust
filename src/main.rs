use std::collections::HashMap;
use std::env;

use rand::prelude::*;
use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

const GRID_X: usize = 10;
const GRID_Y: usize = 10;

const WINDOW_WIDTH: i32 = 900;
const WINDOW_HEIGHT: i32 = 900;

const GRID_OFFSET: i32 = 50;

const BLOCK_SIZE_X: i32 = (WINDOW_WIDTH - 2 * GRID_OFFSET) / 10;
const BLOCK_SIZE_Y: i32 = (WINDOW_HEIGHT - 2 * GRID_OFFSET) / 10;

#[derive(Default, PartialEq)]
enum State {
    #[default]
    START,
    PLAYING,
    GAMEOVER,
    PAUSED,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
enum Direction {
    #[default]
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    fn next(&self) -> Vec2<i32> {
        match self {
            Self::UP => Vec2 { x: 0, y: -1 },
            Self::DOWN => Vec2 { x: 0, y: 1 },
            Self::LEFT => Vec2 { x: -1, y: 0 },
            Self::RIGHT => Vec2 { x: 1, y: 0 },
        }
    }

    fn change(&mut self, rl: &RaylibHandle) {
        if rl.is_key_pressed(KEY_W) {
            if *self != Self::DOWN {
                *self = Self::UP;
            }
        } else if rl.is_key_pressed(KEY_S) {
            if *self != Self::UP {
                *self = Self::DOWN;
            }
        } else if rl.is_key_pressed(KEY_A) {
            if *self != Self::RIGHT {
                *self = Self::LEFT;
            }
        } else if rl.is_key_pressed(KEY_D) {
            if *self != Self::LEFT {
                *self = Self::RIGHT;
            }
        }
    }

    fn rotation(&self) -> i32 {
        match self {
            Self::UP => 90,
            Self::DOWN => 270,
            Self::LEFT => 180,
            Self::RIGHT => 0,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct Vec2<T> {
    x: T,
    y: T,
}

#[derive(Default, Clone, Debug)]
struct SnakeBody {
    pos: Vec2<usize>,
    direction: Direction,
}

impl SnakeBody {
    fn new(x: usize, y: usize, direction: Direction) -> SnakeBody {
        SnakeBody {
            pos: Vec2 { x, y },
            direction,
        }
    }
}

#[derive(Default)]
struct Game {
    snake: Vec<SnakeBody>,
    apple: Vec2<usize>,
    time: f32,
    score: usize,
    snake_body: HashMap<(String, i32), Texture2D>,
    paused: bool,
    next_direction: Direction,
    state: State,
}

impl Game {
    fn new() -> Game {
        Game {
            snake: vec![
                SnakeBody {
                    pos: Vec2 { x: 3, y: 3 },
                    direction: Direction::DOWN,
                },
                SnakeBody {
                    pos: Vec2 { x: 3, y: 4 },
                    direction: Direction::DOWN,
                },
            ],
            apple: Vec2 { x: 3, y: 7 },
            time: 0.0,
            score: 0,
            snake_body: HashMap::new(),
            paused: false,
            next_direction: Direction::DOWN,
            state: State::START,
        }
    }

    fn reset(&mut self) {
        self.snake = vec![
            SnakeBody {
                pos: Vec2 { x: 3, y: 3 },
                direction: Direction::DOWN,
            },
            SnakeBody {
                pos: Vec2 { x: 3, y: 4 },
                direction: Direction::DOWN,
            },
        ];

        self.apple = Vec2 { x: 3, y: 7 };
        self.time = 0.0;
        self.score = 0;
        self.paused = false;
        self.next_direction = Direction::DOWN;
        self.state = State::START;
    }

    fn load_assets(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let assets = vec!["head", "straight_body", "curved_body", "tail"];
        println!("{}", std::env::current_dir().unwrap().display());
        for asset in assets {
            let mut image = Image::load_image(&format!(
                "{}/assets/{}.png",
                env::current_dir().unwrap().display(),
                asset
            ))
            .unwrap();
            image.resize(BLOCK_SIZE_X, BLOCK_SIZE_Y);
            let mut degree = 270;

            for _ in 0..=3 {
                degree = (degree + 90) % 360;
                image.rotate_ccw();
                let texture = rl.load_texture_from_image(&thread, &image).unwrap();
                self.snake_body.insert((asset.to_string(), degree), texture);
            }
        }
    }

    fn update(&mut self, rl: &RaylibHandle) {
        match self.state {
            State::PLAYING | State::PAUSED => {
                self.playing(rl);
            }
            State::START => {}
            State::GAMEOVER => {}
        }
    }

    fn playing(&mut self, rl: &RaylibHandle) {
        let len = self.snake.len() - 1;
        self.next_direction.change(rl);

        if self.time > 0.5 && !self.paused {
            self.time = 0.0;
            self.snake[len].direction = self.next_direction;

            let snake = &self.snake[len];
            let pos = snake.pos;
            let next = snake.direction.next();
            let mut x = pos.x as i32 + next.x;
            let mut y = pos.y as i32 + next.y;

            if x < 0 {
                x = GRID_X as i32 - 1;
            } else if x > GRID_X as i32 - 1 {
                x = 0;
            }

            if y < 0 {
                y = GRID_Y as i32 - 1;
            } else if y > GRID_Y as i32 - 1 {
                y = 0;
            }

            if x as usize == self.apple.x && y as usize == self.apple.y {
                self.score += 100;
                self.snake
                    .push(SnakeBody::new(x as usize, y as usize, snake.direction));
                self.add_apple();
            } else if self.snake_collide(x as usize, y as usize) {
                self.paused = true;
                self.state = State::GAMEOVER;
            } else {
                let mut last_state = self.snake[len].clone();

                self.snake[len].pos.x = x as usize;
                self.snake[len].pos.y = y as usize;

                for index in 1..=len {
                    let temp = self.snake[len - index].clone();
                    self.snake[len - index] = last_state;
                    last_state = temp;
                }
            }
        }

        self.time += rl.get_frame_time();
    }

    fn snake_collide(&self, x: usize, y: usize) -> bool {
        for snake_index in 0..self.snake.len() {
            let snake_pos = self.snake[snake_index].pos;
            if snake_pos.x == x && snake_pos.y == y {
                return true;
            }
        }
        return false;
    }

    fn add_apple(&mut self) {
        let mut rng = rand::thread_rng();
        let mut x = (rng.gen::<f32>() * GRID_X as f32) as usize;
        let mut y = (rng.gen::<f32>() * GRID_X as f32) as usize;

        while self.snake_collide(x, y) {
            x = (rng.gen::<f32>() * GRID_X as f32) as usize;
            y = (rng.gen::<f32>() * GRID_X as f32) as usize;
        }

        self.apple = Vec2 { x, y };
    }

    fn render(&self, render: &mut RaylibDrawHandle) {
        match self.state {
            State::START => {
                render.draw_text(
                    &format!("LET'S PLAY"),
                    (WINDOW_WIDTH / 2) - 300,
                    (WINDOW_HEIGHT / 2) as i32,
                    100,
                    Color::GRAY,
                );

                render.draw_text(
                    &format!("Press SPACE to PLAY"),
                    (WINDOW_WIDTH / 2) - 300,
                    (WINDOW_HEIGHT / 2) + 100,
                    50,
                    Color::GRAY,
                );
            }
            State::GAMEOVER => {
                render.draw_text(
                    &format!("GAME OVER!"),
                    GRID_OFFSET,
                    (WINDOW_HEIGHT / 2) as i32,
                    100,
                    Color::GRAY,
                );

                render.draw_text(
                    &format!("Press SPACE to PLAY AGAIN"),
                    GRID_OFFSET,
                    (WINDOW_HEIGHT / 2) + 100,
                    50,
                    Color::GRAY,
                );
            }
            _ => {
                let width: i32 = GRID_X as i32 * BLOCK_SIZE_X;
                let height: i32 = GRID_Y as i32 * BLOCK_SIZE_Y;

                for y in 0..=GRID_Y {
                    for x in 0..=GRID_X {
                        render.draw_line(
                            GRID_OFFSET + x as i32 * BLOCK_SIZE_X,
                            GRID_OFFSET + y as i32 * BLOCK_SIZE_Y,
                            width,
                            GRID_OFFSET + y as i32 * BLOCK_SIZE_Y,
                            Color::BLACK,
                        );

                        render.draw_line(
                            GRID_OFFSET + x as i32 * BLOCK_SIZE_X,
                            GRID_OFFSET + y as i32 * BLOCK_SIZE_Y,
                            GRID_OFFSET + x as i32 * BLOCK_SIZE_X,
                            height,
                            Color::BLACK,
                        );
                    }
                }

                for y in 0..GRID_Y {
                    for x in 0..GRID_X {
                        render.draw_rectangle(
                            GRID_OFFSET + x as i32 * BLOCK_SIZE_X,
                            GRID_OFFSET + 1 + y as i32 * BLOCK_SIZE_Y,
                            BLOCK_SIZE_X - 1,
                            BLOCK_SIZE_Y - 1,
                            Color::WHITE,
                        );
                    }
                }

                render.draw_rectangle(
                    GRID_OFFSET + self.apple.x as i32 * BLOCK_SIZE_X,
                    GRID_OFFSET + 1 + self.apple.y as i32 * BLOCK_SIZE_Y,
                    BLOCK_SIZE_X - 1,
                    BLOCK_SIZE_Y - 1,
                    Color::RED,
                );

                let len = self.snake.len() - 1;

                render.draw_texture(
                    &self
                        .snake_body
                        .get(&("head".to_string(), self.snake[len].direction.rotation()))
                        .unwrap(),
                    GRID_OFFSET + self.snake[len].pos.x as i32 * BLOCK_SIZE_X,
                    GRID_OFFSET + 1 + self.snake[len].pos.y as i32 * BLOCK_SIZE_Y,
                    Color::LIGHTGRAY,
                );

                render.draw_texture(
                    &self
                        .snake_body
                        .get(&("tail".to_string(), self.snake[0].direction.rotation()))
                        .unwrap(),
                    GRID_OFFSET + self.snake[0].pos.x as i32 * BLOCK_SIZE_X,
                    GRID_OFFSET + 1 + self.snake[0].pos.y as i32 * BLOCK_SIZE_Y,
                    Color::LIGHTGRAY,
                );

                for snake_index in 1..len {
                    let (body_part, rotation) = self.get_snake_body(snake_index);

                    render.draw_texture(
                        &self.snake_body.get(&(body_part, rotation)).unwrap(),
                        GRID_OFFSET + self.snake[snake_index].pos.x as i32 * BLOCK_SIZE_X,
                        GRID_OFFSET + 1 + self.snake[snake_index].pos.y as i32 * BLOCK_SIZE_Y,
                        Color::LIGHTGRAY,
                    );
                }

                render.draw_text(
                    &format!("Score: {}", self.score),
                    GRID_OFFSET,
                    (GRID_OFFSET / 2) as i32,
                    20,
                    Color::GREEN,
                );

                if self.state == State::PAUSED {
                    render.draw_text(
                        &format!("PAUSED"),
                        (WINDOW_WIDTH / 2) - 300,
                        (WINDOW_HEIGHT / 2) as i32,
                        100,
                        Color::GRAY,
                    );

                    render.draw_text(
                        &format!("Press SPACE to PLAY"),
                        (WINDOW_WIDTH / 2) - 300,
                        (WINDOW_HEIGHT / 2) + 100,
                        50,
                        Color::GRAY,
                    );
                }
            }
        }
    }

    fn get_snake_body(&self, snake_index: usize) -> (String, i32) {
        let before = self.snake[snake_index - 1].pos;
        let after = self.snake[snake_index + 1].pos;
        let snake = self.snake[snake_index].pos;
        let len_x = GRID_X - 1;
        let len_y = GRID_Y - 1;

        let left = if snake.x as i32 - 1 < 0 {
            len_x
        } else {
            snake.x - 1
        };

        let right = if snake.x + 1 > len_x { 0 } else { snake.x + 1 };

        let up = if snake.y as i32 - 1 < 0 {
            len_y
        } else {
            snake.y - 1
        };

        let down = if snake.y + 1 > len_y { 0 } else { snake.y + 1 };

        if (up == after.y && right == before.x) || (up == before.y && right == after.x) {
            ("curved_body".to_string(), 0)
        } else if (up == after.y && left == before.x) || (up == before.y && left == after.x) {
            ("curved_body".to_string(), 90)
        } else if (down == after.y && left == before.x) || (down == before.y && left == after.x) {
            ("curved_body".to_string(), 180)
        } else if (down == after.y && right == before.x) || (down == before.y && right == after.x) {
            ("curved_body".to_string(), 270)
        } else {
            (
                "straight_body".to_string(),
                self.snake[snake_index].direction.rotation(),
            )
        }
    }

    // DEBUG
    fn hover(&self, render: &mut RaylibDrawHandle, mouse_at: Vector2) {
        let mouse_x = mouse_at.x as usize;
        let mouse_y = mouse_at.y as usize;

        for snake_index in 0..self.snake.len() {
            let snake_x = (GRID_OFFSET as usize
                + self.snake[snake_index].pos.x * BLOCK_SIZE_X as usize)
                as usize;
            let snake_y = (GRID_OFFSET as usize
                + self.snake[snake_index].pos.y * BLOCK_SIZE_Y as usize)
                as usize;

            if mouse_x > snake_x
                && mouse_x < snake_x + BLOCK_SIZE_X as usize
                && mouse_y > snake_y
                && mouse_y < snake_y + BLOCK_SIZE_Y as usize
            {
                if snake_index > 0 && snake_index < self.snake.len() - 1 {
                    let (body_part, rotation) = self.get_snake_body(snake_index);

                    render.draw_text(
                        &format!(
                            "cur_direction: {:?}\nbody_part: {}\nrotation: {}",
                            self.snake[snake_index].direction, body_part, rotation
                        ),
                        GRID_OFFSET,
                        GRID_OFFSET,
                        30,
                        Color::RED,
                    );
                    render.draw_text(
                        &format!("before: {:?}\n", self.snake[snake_index - 1].pos),
                        GRID_OFFSET,
                        GRID_OFFSET + 150,
                        30,
                        Color::BLUE,
                    );
                    render.draw_text(
                        &format!("current: {:?}\n", self.snake[snake_index].pos),
                        GRID_OFFSET,
                        GRID_OFFSET + 250,
                        30,
                        Color::GREEN,
                    );
                    render.draw_text(
                        &format!("after: {:?}\n", self.snake[snake_index + 1].pos),
                        GRID_OFFSET,
                        GRID_OFFSET + 350,
                        30,
                        Color::BLUE,
                    );
                } else {
                    render.draw_text(
                        &format!(
                            "cur_direction: {:?}\nrotation: {}",
                            self.snake[snake_index].direction,
                            self.snake[snake_index].direction.rotation()
                        ),
                        GRID_OFFSET,
                        GRID_OFFSET,
                        30,
                        Color::RED,
                    );
                }
            }
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("snake game")
        .build();

    let mut game = Game::new();
    game.load_assets(&mut rl, &thread);

    while !rl.window_should_close() {
        if rl.is_key_pressed(KEY_SPACE) {
            match game.state {
                State::PLAYING | State::PAUSED => {
                    game.paused = !game.paused;
                    game.state = if game.paused {
                        State::PAUSED
                    } else {
                        State::PLAYING
                    };
                }
                State::START => {
                    game.state = State::PLAYING;
                }
                State::GAMEOVER => game.reset(),
            }
        }

        let mouse_at = rl.get_mouse_position();

        game.update(&rl);

        let mut render = rl.begin_drawing(&thread);

        render.clear_background(Color::WHITE);
        game.render(&mut render);
        game.hover(&mut render, mouse_at);
    }
}
