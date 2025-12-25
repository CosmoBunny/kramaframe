use std::error::Error;
use std::io::{self, Stdout};
use std::ops::{Add, Mul, Sub};
use std::time::{Duration, Instant};

use cand::{Logger, MultiLogger, StatusLevel, StorageProvider, black_box_cand};
use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use kramaframe::keylist::TRES16Bits;
use kramaframe::prelude::{KeyFrameFunction, KeyList};
use kramaframe::{BTclasslist, BTframelist, KramaFrame};
use ratatui::crossterm::{self, execute};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

fn main() {
    let mut logger = MultiLogger(Instant::now(), ErrorConv);
    black_box_cand!(Logger(Instant::now(), ErrorConv));

    let mut terminal = ratatui::init();
    let mut app = AnimationRack::default();
    // Raw Mode
    logger.try_run(enable_raw_mode().map_err(|err| err.into()));
    let mut stdout = io::stdout();
    logger.try_run(execute!(stdout, EnterAlternateScreen).map_err(|e| e.into()));

    app.run(&mut terminal, &mut logger);

    logger.try_run(execute!(stdout, LeaveAlternateScreen).map_err(|e| e.into()));
    logger.try_run(disable_raw_mode().map_err(|e| e.into()));
    ratatui::restore();
}

// This struct will use to control the animation of the rack
// like changing duration, changing range, chan
struct AnimationRack {
    // 16 bit timing 0 to 65535 milliseconds, for loseless accuracy. 16 bits progress precision.
    animation: KramaFrame<BTclasslist, BTframelist<TRES16Bits, i16>>,
    // response rate (ms) = (1 / frame_rate) * 1000 AKA expected time between frames.
    res_rate_ms: u8,
    // Sets of all KeyFrameFunction Same length as duration_sets.
    lag: f32,
    // quit signal
    quit: bool,
    // Tab
    tab: Tab,
    // upper widget
    upper: &'static str,
    // lower widget
    lower: &'static str,
}

impl Default for AnimationRack {
    fn default() -> Self {
        let mut animation = KramaFrame::default();
        animation.extend_iter_classlist([
            (
                "slide",
                KeyFrameFunction::new_cubic_bezier_f32(1., 0.0, 0.6, 1.),
            ),
            ("label", KeyFrameFunction::EaseInOut),
            ("linear", KeyFrameFunction::Linear),
            ("ease", KeyFrameFunction::Ease),
            ("easein", KeyFrameFunction::EaseIn),
            ("easeout", KeyFrameFunction::EaseOut),
            ("easeinout", KeyFrameFunction::EaseInOut),
            (
                "cubic",
                KeyFrameFunction::new_cubic_bezier_f32(1., 0.0, 0.6, 1.),
            ),
            ("quadratic", KeyFrameFunction::Quadratic),
        ]);
        animation.framelist.extend([
            ("slide", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("label", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("linear", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("easein", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("easeout", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("easeinout", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("ease", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("cubic", KeyList::new(0, TRES16Bits::from_millis(1000))),
            ("quadratic", KeyList::new(0, TRES16Bits::from_millis(1000))),
        ]);
        Self {
            animation,
            // By default nearly 60 FPS
            res_rate_ms: 16,
            // lag in ms
            lag: 0.0,
            quit: false,
            tab: Tab {
                menu: vec![
                    "label",
                    "linear",
                    "ease",
                    "easein",
                    "easeout",
                    "easeinout",
                    "cubic",
                    "quadratic",
                ],
                curr: 0,
            },
            upper: "label",
            lower: "linear",
        }
    }
}

impl AnimationRack {
    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        logger: &mut MultiLogger<Instant, ErrorConv>,
    ) {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(self.res_rate_ms as u64);

        // Main running app loop
        while !self.quit {
            if let Some(message) = self.view(terminal, logger) {
                self.update(message);
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            let (msg, _) = logger.try_get(self.handler(timeout), |_| {});
            if let Some(msg) = msg {
                self.update(msg);
            }

            let elapsed = last_tick.elapsed();
            if elapsed >= tick_rate {
                self.lag = (elapsed.as_millis() as i64 - tick_rate.as_millis() as i64) as f32;
                self.animation
                    .update_progress(TRES16Bits::from_millis(elapsed.as_millis() as u16));
                self.permanet_animation();
                last_tick = Instant::now();
            }
        }
    }
    fn view(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        logger: &mut MultiLogger<Instant, ErrorConv>,
    ) -> Option<Message> {
        let (size, mut logger) = logger.try_get(terminal.size().map_err(|e| e.into()), |_| {});
        // Height should be more than 20
        let height = size.height;
        let width = size.width;

        let warning_color = self.animation.get_generic_byrange(
            "label",
            0,
            AnimatingColor(Color::Rgb(255, 0, 0))..(AnimatingColor(Color::Rgb(255, 255, 0))),
        );

        let active = self.animation.get_generic_byrange(
            "label",
            0,
            AnimatingColor(Color::Rgb(2, 230, 255))..(AnimatingColor(Color::Rgb(255, 255, 255))),
        );

        // helper Dialog to show keybind

        if height <= 25 || width <= 40 {
            logger.try_run(
                terminal
                    .draw(|f| {
                        f.render_widget(
                            Paragraph::new(format!("Height {} > 25\nWidth {} > 40", height, width))
                                .fg(warning_color.0),
                            f.area(),
                        );
                    })
                    .map_err(|e| e.into()),
            );
            return None;
        }
        let lag_label = format!("Lag: {:.2} ms", self.lag);
        let kramaframe_label = "⎣ＫｒａｍａＦｒａｍｅ⎦";
        // Header should have Two Div with Single height.
        let divider = Layout::new(
            Direction::Vertical,
            [Constraint::Length(2), Constraint::Length(height - 2)],
        );
        let header = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(width - lag_label.len() as u16),
                Constraint::Length(lag_label.len() as u16 + 1),
            ],
        );

        let menu_length = self
            .tab
            .menu
            .iter()
            .map(|x| x.len() as u16 + 2)
            .max()
            .unwrap_or(0);

        let main_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(width - menu_length),
                Constraint::Length(menu_length),
            ],
        );

        let menu_layout = Layout::new(
            Direction::Vertical,
            self.tab.menu.iter().map(|_| Constraint::Length(3)),
        );

        logger.try_run(
            terminal
                .draw(|f| {
                    let divider_area = divider.split(f.area());
                    let header_area = header.split(divider_area[0]);
                    let main_area = main_layout.split(divider_area[1]);
                    let menu_area = menu_layout.split(main_area[1]);
                    //
                    f.render_widget(
                        Paragraph::new(kramaframe_label)
                            .fg(Color::Green)
                            .block(Block::bordered().borders(Borders::BOTTOM)),
                        header_area[0],
                    );
                    f.render_widget(
                        Paragraph::new(lag_label)
                            .fg(Color::Green)
                            .block(Block::bordered().borders(Borders::BOTTOM | Borders::LEFT)),
                        header_area[1],
                    );
                    // list all menu
                    for (index, name) in self.tab.menu.iter().enumerate() {
                        f.render_widget(
                            Paragraph::new(*name)
                                .fg(if index == self.tab.curr {
                                    active.0
                                } else {
                                    Color::DarkGray
                                })
                                .block(Block::bordered()),
                            menu_area[index],
                        );
                    }
                    // slide animation but height of menu_area[1].
                    let main_slide_division = Layout::new(
                        Direction::Vertical,
                        [
                            Constraint::Length(self.animation.get_value_byrange_inclusive(
                                "slide",
                                0,
                                main_area[0].height..=0,
                            )),
                            Constraint::Length(self.animation.get_value_byrange_inclusive(
                                "slide",
                                0,
                                0..=main_area[0].height,
                            )),
                        ],
                    );

                    let main_body_area = main_slide_division.spacing(1).split(main_area[0]);
                    // main body
                    f.render_widget(
                        Container {
                            slide: self.animation.get_value_byrange_inclusive(
                                self.upper,
                                0,
                                0f32..=1f32,
                            ),
                            class: self.upper,
                            duration: self.animation.get_timing(self.upper, 0).0,
                        },
                        main_body_area[0],
                    );
                    f.render_widget(
                        Container {
                            slide: self.animation.get_value_byrange_inclusive(
                                self.lower,
                                0,
                                0f32..=1f32,
                            ),
                            class: self.lower,
                            duration: self.animation.get_timing(self.lower, 0).0,
                        },
                        main_body_area[1],
                    );

                    //
                })
                .map_err(|e| e.into()),
        );

        None
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Exit => {
                self.quit = true;
            }
            Message::TabDown => {
                if self.tab.menu.len() - 1 == self.tab.curr {
                    self.tab.curr = 0;
                } else {
                    self.tab.curr += 1;
                }
                self.animation.restart_progress("slide", 0);
                self.lower = self.tab.menu[self.tab.curr];
                self.upper = self.tab.menu[if self.tab.curr == 0 {
                    self.tab.menu.len() - 1
                } else {
                    self.tab.curr - 1
                }];
            }
            Message::TabUp => {
                if self.tab.curr == 0 {
                    self.tab.curr = self.tab.menu.len() - 1;
                } else {
                    self.tab.curr -= 1;
                }
                self.animation.reverse_start("slide", 0);
                self.upper = self.tab.menu[self.tab.curr];
                self.lower = self.tab.menu[if self.tab.curr == self.tab.menu.len() - 1 {
                    0
                } else {
                    self.tab.curr + 1
                }];
            }
            Message::Increment => {
                let timing = self.animation.get_timing(self.tab.current(), 0).0 + 1;
                self.animation
                    .set_timing(self.tab.current(), 0, TRES16Bits(timing));
            }
            Message::Decrement => {
                let timing = self.animation.get_timing(self.tab.current(), 0).0 - 1;
                self.animation
                    .set_timing(self.tab.current(), 0, TRES16Bits(timing));
            }
            _ => {}
        }
    }
    fn permanet_animation(&mut self) {
        macro_rules! anim_control {
            ($anim:expr, $label:expr) => {
                if $anim.get_progress_f32($label, 0) == 1.0 {
                    $anim.reverse_animate($label, 0);
                }
                if $anim.get_progress_f32($label, 0) == 0.0 {
                    $anim.restart_progress($label, 0);
                }
            };
        }
        anim_control!(self.animation, "label");
        anim_control!(self.animation, "linear");
        anim_control!(self.animation, "ease");
        anim_control!(self.animation, "easein");
        anim_control!(self.animation, "easeout");
        anim_control!(self.animation, "easeinout");
        anim_control!(self.animation, "cubic");
        anim_control!(self.animation, "quadratic");
    }

    fn handler(&mut self, timeout: Duration) -> Result<Option<Message>, Box<dyn Error>> {
        // Check if an event is available within the timeout
        if event::poll(timeout)? {
            match event::read()? {
                event::Event::Resize(_, _) => Ok(Some(Message::Update)),
                event::Event::Key(key) => {
                    if key.code == KeyCode::Char('q') {
                        Ok(Some(Message::Exit))
                    } else if key.code == KeyCode::Down {
                        Ok(Some(Message::TabDown))
                    } else if key.code == KeyCode::Up {
                        Ok(Some(Message::TabUp))
                    } else if key.code == KeyCode::Char('w') {
                        Ok(Some(Message::Increment))
                    } else if key.code == KeyCode::Char('s') {
                        Ok(Some(Message::Decrement))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            // No event available, return immediately without blocking
            Ok(None)
        }
    }
}

enum Message {
    TabUp,
    TabDown,
    Increment,
    Decrement,
    Update,
    Exit,
}

struct Tab {
    menu: Vec<&'static str>,
    curr: usize,
}

impl Tab {
    fn current(&self) -> &'static str {
        self.menu[self.curr]
    }
}

use ratatui::widgets::Widget;

struct Container {
    // ranging 0.0 to 1.0
    slide: f32,
    class: &'static str,
    duration: u16,
}

impl Widget for Container {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text_width = self.class.len() as u16;
        let page_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(area.width - text_width),
                Constraint::Length(text_width),
            ],
        );
        let fill = (area.height - 2) as f32 * self.slide;

        // █▇▆▅▄▃▂  10/8 = 1.25
        // 10   -> █
        // 8.75 -> ▇
        // 7.5  -> ▆
        // 6.25 -> ▅
        // 5    -> ▄
        // 3.75 -> ▃
        // 2.5  -> ▂
        // 1.25 -> ▁
        // let 68 , 6 full block █ and remainder is 8 which near 7.5 but take 8.75 as roundup to get color factoring = 8.75 - 8 = 0.75.

        let no_of_full_blocks = fill.floor() as u16;
        let last_block = match fill - no_of_full_blocks as f32 {
            x @ 0.875..1.00 => ("█████", 10. - x),
            x @ 0.75..0.875 => ("▇▇▇▇▇", 8.75 - x),
            x @ 0.625..0.75 => ("▆▆▆▆▆", 7.5 - x),
            x @ 0.50..0.625 => ("▅▅▅▅▅", 6.25 - x),
            x @ 0.375..0.50 => ("▄▄▄▄▄", 5.0 - x),
            x @ 0.25..0.375 => ("▃▃▃▃▃", 3.75 - x),
            x @ 0.125..0.25 => ("▂▂▂▂▂", 2.5 - x),
            x @ 0.0..0.125 => ("▁▁▁▁▁", 1.25 - x),
            _ => (" ", 0.0),
        };
        let page = page_layout.margin(1).split(area);
        let max_height = page[0].height;

        let mut lines = Vec::new();
        for _ in 0..no_of_full_blocks.min(max_height) {
            lines.push(Line::raw("█████").fg(Color::Rgb(10, 250, 255)));
        }

        let color_ratio = AnimatingColor(Color::Rgb(10, 250, 255)) * (last_block.1 / 1.25);
        if lines.len() < max_height as usize {
            lines.push(Line::raw(last_block.0).fg(color_ratio.0));
        }

        let material = Layout::new(
            Direction::Horizontal,
            [Constraint::Length(6), Constraint::Length(page[0].width - 6)],
        )
        .split(page[0]);
        // fill empty remaining space
        for _ in 0..max_height.saturating_sub(lines.len() as u16) {
            lines.push(Line::raw("     ").fg(Color::Rgb(10, 250, 255)));
        }

        lines.reverse();

        Paragraph::new("")
            .block(Block::bordered())
            .render(area, buf);

        Paragraph::new(lines).render(material[0], buf);

        Paragraph::new(self.class).render(page[1], buf);
        Paragraph::new(format!(
            "█████\n█████\n█████\n
Key: w - increment of timing
Key: s - decrement of timing
Animation Duration: {} milliseconds
            ",
            self.duration
        ))
        .fg((AnimatingColor(Color::Rgb(255, 255, 255)) * self.slide).0)
        .render(material[1], buf);
    }
}

#[derive(Clone)]
pub struct ErrorConv;

impl StorageProvider for ErrorConv {
    fn write_data(&mut self, args: std::fmt::Arguments, debuglevel: &cand::StatusLevel) {
        if matches!(debuglevel, StatusLevel::Error | StatusLevel::Critical) {
            ratatui::restore();
            let _ = disable_raw_mode().is_ok();
        }
        println!("{}", args)
    }
}

// implement Add, Sub, Mul to use get_value_byrange
#[derive(Copy, Clone)]
struct AnimatingColor(Color);
impl Add for AnimatingColor {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self.0, rhs.0) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                AnimatingColor(Color::Rgb(r1 + r2, g1 + g2, b1 + b2))
            }
            (s, _) => AnimatingColor(s),
        }
    }
}

impl Sub for AnimatingColor {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self.0, rhs.0) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r_diff = (r1 as i16 - r2 as i16).abs() as u8;
                let g_diff = (g1 as i16 - g2 as i16).abs() as u8;
                let b_diff = (b1 as i16 - b2 as i16).abs() as u8;
                AnimatingColor(Color::Rgb(r_diff, g_diff, b_diff))
            }
            (s, _) => AnimatingColor(s),
        }
    }
}

impl Mul<f32> for AnimatingColor {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        // rhs should range from 0.0 to 1.0
        let rhs = rhs.max(0.0).min(1.0);
        match self.0 {
            Color::Rgb(r, g, b) => AnimatingColor(Color::Rgb(
                (r as f32 * rhs) as u8,
                (g as f32 * rhs) as u8,
                (b as f32 * rhs) as u8,
            )),
            _ => self,
        }
    }
}
