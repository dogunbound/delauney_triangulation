use delauney_triangulation::DelauneyTriangulationInformation;
use sfml::{
    graphics::{Color, RcFont, RcText, RenderStates, RenderTarget, RenderWindow, View},
    system::Vector2f,
    window::{mouse::Button, Event, Key, Style},
};

pub mod circle;
pub mod delauney_triangulation;
pub mod math;
pub mod utils;

fn setup_window() -> RenderWindow {
    let mut window = RenderWindow::new(
        (1280, 720),
        "Delauney Triangulation",
        Style::DEFAULT,
        &Default::default(),
    );
    window.set_vertical_sync_enabled(true);

    window
}

fn load_font() -> RcFont {
    let mut font = RcFont::from_file("m6x11.ttf").unwrap_or_else(|| {
        const FAILED_TO_LOAD_FONT_MESSAGE: &str = "Failed to load m6x11.ttf. Ensure the ttf is in the same directory you are calling the executable.";
        panic!("{}", FAILED_TO_LOAD_FONT_MESSAGE);
    });
    font.set_smooth(false);

    font
}

fn display_text(window: &mut RenderWindow, all_text_on_window: &[RcText]) {
    let rs = RenderStates::default();
    for text in all_text_on_window {
        window.draw_rc_text(&text, &rs);
    }
}

const CHARACTER_SIZE: u32 = 24;
fn setup_text(font: &RcFont) -> Vec<RcText> {
    const DIRECTIONS_TEXT: &str = "Click anywhere on screen to add vertices

<Space> to start delauney triangulation
<Space> to pause animation (if started and running)
<Space> to continue animation (if paused)
<Esc> to stop animation
<f> to make animation faster
<s> to make animation slower
<c> to go frame by frame (if paused)
<r> to remove all vertices
<h> to hide/show help text";

    let directions = RcText::new(DIRECTIONS_TEXT, font, CHARACTER_SIZE);

    vec![directions]
}

const FRAME_DURATION_INCREMENT_DECREMENT_AMOUNT: u8 = 2;
fn main() {
    let mut window = setup_window();
    let font = load_font();
    let mut all_text_on_window = setup_text(&font);
    let mut vertices = vec![];
    let (mut is_animating, mut is_paused) = (false, true);
    let (mut num_of_frames_since_last_calculation, mut frame_duration_between_calculations): (
        u32,
        u32,
    ) = (0, 4);
    let mut delauney_triangulation_information = DelauneyTriangulationInformation::default();
    let mut hide_help_text = false;

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::Resized { width, height } => {
                    let (width, height) = (width as f32, height as f32);
                    let size = Vector2f::new(width, height);
                    let center = size - size / 2.;
                    window.set_view(&View::new(center, size));
                    for text in &mut all_text_on_window {
                        text.set_character_size((width / 1280. * CHARACTER_SIZE as f32) as u32);
                    }
                }
                Event::KeyPressed { code, .. } => match code {
                    Key::Space => {
                        if !is_animating {
                            delauney_triangulation_information.set_point_list(vertices.clone());
                        }

                        is_animating = true;
                        is_paused = !is_paused;
                    }
                    Key::Escape => {
                        is_animating = false;
                        is_paused = true;
                        delauney_triangulation_information.reset_delauney_mesh();
                    }
                    Key::F => {
                        frame_duration_between_calculations = frame_duration_between_calculations
                            .saturating_sub(FRAME_DURATION_INCREMENT_DECREMENT_AMOUNT.into());

                        if frame_duration_between_calculations == 0 {
                            frame_duration_between_calculations = 1;
                        }
                    }
                    Key::S => {
                        frame_duration_between_calculations = frame_duration_between_calculations
                            .saturating_add(FRAME_DURATION_INCREMENT_DECREMENT_AMOUNT.into())
                    }
                    Key::C => {
                        num_of_frames_since_last_calculation =
                            frame_duration_between_calculations + 1
                    }
                    Key::H => {
                        hide_help_text = !hide_help_text;
                    }
                    Key::R => {
                        vertices = vec![];
                    }
                    _ => {}
                },
                Event::MouseButtonPressed { button, x, y } => {
                    if button == Button::Left && !is_animating {
                        vertices.push(Vector2f::new(x as f32, y as f32));
                    }
                }
                _ => {}
            }
        }

        if num_of_frames_since_last_calculation >= frame_duration_between_calculations {
            delauney_triangulation_information.update_triangulation();
            num_of_frames_since_last_calculation = 0;
        }

        window.clear(Color::rgb(10, 10, 10));
        if is_animating {
            delauney_triangulation_information.draw(&mut window);
        } else {
            utils::display_vertices_as_small_yellow_circles(&mut window, &vertices);
        }

        if !hide_help_text {
            display_text(&mut window, &all_text_on_window);
        }
        window.display();

        if !is_paused {
            num_of_frames_since_last_calculation += 1;
        }
    }
}
