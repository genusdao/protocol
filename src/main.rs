use candle_core::DType;
use candle_core::{Device, Tensor};
use ggez::event;
use ggez::event::EventHandler;
use ggez::graphics;
use ggez::{Context, ContextBuilder, GameResult};
use rand::Rng;

/// Config for the start of the game
#[derive(Debug, Clone)]
pub struct Config {
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: f32,
    pub fps: u32,
}

struct MainState {
    config: Config,
    cells: Vec<f64>,
}

impl MainState {
    pub fn new(config: Config, initial_state: Vec<f64>) -> Self {
        MainState {
            config,
            cells: initial_state,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.config.fps) {
            let image = Tensor::from_vec(
                self.cells.clone(),
                (1, 1, self.config.grid_width, self.config.grid_height),
                &Device::Cpu,
            )
            .unwrap();

            let filter = [[1., 1., 1.], [1., 0., 1.], [1., 1., 1.]];
            let kernel = Tensor::new(&[[filter]], &Device::Cpu).unwrap();
            let res = image.conv2d(&kernel, 1, 1, 1, 1).unwrap();

            let res_flatten = res.flatten_to(3).unwrap().to_vec1::<f64>().unwrap();
            let cells_grown = self
                .cells
                .iter()
                .zip(res_flatten)
                .map(|(x, y)| x + f64::from(y == 3.) - f64::from((y < 2.) | (y > 3.)))
                .collect();

            let zeros = Tensor::zeros(
                (self.config.grid_width, self.config.grid_height),
                DType::F64,
                &Device::Cpu,
            )
            .unwrap();
            let one = Tensor::ones(
                (self.config.grid_width, self.config.grid_height),
                DType::F64,
                &Device::Cpu,
            )
            .unwrap();
            let grown_tensor = Tensor::from_vec(
                cells_grown,
                (self.config.grid_width, self.config.grid_height),
                &Device::Cpu,
            )
            .unwrap();

            let res = grown_tensor.clamp(&zeros, &one).unwrap();
            let res_flatten = res.flatten_all().unwrap().to_vec1().unwrap();
            self.cells = res_flatten;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        let mut builder = graphics::MeshBuilder::new();

        // Draw cells
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, x)| **x > 0.)
            .for_each(|(i, _)| {
                let pos_x = i % self.config.grid_width;
                let pos_y = i / self.config.grid_height;
                let color = graphics::Color::GREEN; // Green
                let draw_mode = graphics::DrawMode::fill();
                let rect = graphics::Rect::new(
                    pos_x as f32 * self.config.cell_size,
                    pos_y as f32 * self.config.cell_size,
                    self.config.cell_size,
                    self.config.cell_size,
                );
                builder.rectangle(draw_mode, rect, color).unwrap();
            });

        let mesh = builder.build();
        let mesh = graphics::Mesh::from_data(ctx, mesh);
        canvas.draw(&mesh, graphics::DrawParam::default());
        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let screen_size = (1000., 1000.);
    let grid_size = (100, 100);
    let cell_size = 10.;
    let fps = 20;

    let mut rng = rand::thread_rng();
    let initial_state = (0..grid_size.0 * grid_size.1)
        .into_iter()
        .map(|_| rng.gen::<bool>().into())
        .collect::<Vec<f64>>();

    // Set configuration
    let config: Config = Config {
        grid_width: grid_size.0,
        grid_height: grid_size.1,
        cell_size,
        fps,
    };
    let state = MainState::new(config, initial_state);

    // Setup ggez stuff
    let cb = ContextBuilder::new("game_of_life", "Zoran")
        .window_mode(ggez::conf::WindowMode::default().dimensions(screen_size.0, screen_size.1));
    let (ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("Game of life");
    // Setup game state -> game loop
    event::run(ctx, event_loop, state)
}
