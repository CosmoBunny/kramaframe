// NOTE : This code is mostly generate by AI but improved by human. if you see something that can more improve possible pls open PR/Commit withour hesitation. - Thanks for Reading :)

use std::error::Error;
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cand::{Logger, MultiLogger, StatusLevel, StorageProvider, black_box_cand};
use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use kramaframe::keylist::TRES16Bits;
use kramaframe::microcl::UClassList;
use kramaframe::microfl::UFrameList;
use kramaframe::{KramaFrame, ukramaframe};
use ratatui::crossterm::{self, execute};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ringbuf::{Consumer, HeapRb, Producer};
use rustfft::{FftPlanner, num_complex::Complex, num_traits::Zero};

const FFT_SIZE: usize = 2048;
// Increased bands to 60 for finer resolution
const NUM_BANDS: usize = 60;

#[derive(Debug)]
struct StringError(String);
impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for StringError {}

fn main() {
    let mut logger = MultiLogger(Instant::now(), ErrorConv);
    black_box_cand!(Logger(Instant::now(), ErrorConv));

    let args: Vec<String> = std::env::args().collect();
    let host = cpal::default_host();

    if args.contains(&"--list".to_string()) {
        println!("Available Input Devices:");
        if let Ok(devices) = host.input_devices() {
            for (i, device) in devices.enumerate() {
                println!("{}: {}", i, device.name().unwrap_or("Unknown".into()));
            }
        }
        return;
    }

    let device = if args.len() > 1 {
        if let Ok(idx) = args[1].parse::<usize>() {
            host.input_devices().ok().and_then(|mut d| d.nth(idx))
        } else {
            host.default_input_device()
        }
    } else {
        host.default_input_device()
    };

    let device = match device {
        Some(d) => d,
        None => {
            println!("No input device found. Use --list to see available devices.");
            return;
        }
    };

    let device_name = device.name().unwrap_or("Unknown".into());

    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            logger.try_run::<()>(Err(StringError(format!(
                "Failed to get default input config: {}",
                e
            ))
            .into()));
            return;
        }
    };

    // LOCK-FREE RING BUFFER setup
    let rb_l = HeapRb::<f32>::new(8192);
    let rb_r = HeapRb::<f32>::new(8192);
    let (prod_l, cons_l) = rb_l.split();
    let (prod_r, cons_r) = rb_r.split();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            run_stream::<f32>(&device, &config.clone().into(), prod_l, prod_r, &mut logger)
        }
        cpal::SampleFormat::I16 => {
            run_stream::<i16>(&device, &config.clone().into(), prod_l, prod_r, &mut logger)
        }
        cpal::SampleFormat::U16 => {
            run_stream::<u16>(&device, &config.clone().into(), prod_l, prod_r, &mut logger)
        }
        _ => {
            logger.try_run::<()>(Err(StringError("Unsupported sample format".into()).into()));
            return;
        }
    };

    let _stream = match stream {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Err(e) = _stream.play() {
        logger.try_run::<()>(Err(
            StringError(format!("Failed to play stream: {}", e)).into()
        ));
        return;
    }

    // Linux-specific: Auto-connect to monitor source if possible
    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(500));
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(
                    r#"
                    TARGET=$(pactl list short sources | grep "monitor" | grep "RUNNING" | awk '{print $1}' | head -n 1);
                    if [ -z "$TARGET" ]; then TARGET=$(pactl list short sources | grep "monitor" | awk '{print $1}' | head -n 1); fi;
                    
                    if [ -n "$TARGET" ]; then
                        pactl list short source-outputs | while read -r line; do
                            STREAM_ID=$(echo "$line" | awk '{print $1}')
                            pactl move-source-output "$STREAM_ID" "$TARGET" >/dev/null 2>&1
                        done
                    fi
                "#,
                )
                .output();
        });
    }

    logger.try_run(
        enable_raw_mode()
            .map_err(|e| StringError(format!("Failed to enable raw mode: {}", e)).into()),
    );
    let mut stdout = io::stdout();
    logger.try_run(
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| StringError(format!("Failed to enter alternate screen: {}", e)).into()),
    );
    let mut terminal = ratatui::init();

    // Single animation class with 140ms default duration
    // IDs Optimized: Using just ID 0 for global sync to reduce overhead
    let animation = ukramaframe!(<TRES16Bits, i16, u8>
        "viz" EaseOut [0] 0.14 s;
    );

    let mut app = AudioApp::new(animation, cons_l, cons_r, device_name);
    app.run(&mut terminal, &mut logger);

    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    ratatui::restore();
}

fn run_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut prod_l: Producer<f32, Arc<ringbuf::HeapRb<f32>>>,
    mut prod_r: Producer<f32, Arc<ringbuf::HeapRb<f32>>>,
    logger: &mut MultiLogger<Instant, ErrorConv>,
) -> Result<cpal::Stream, ()>
where
    T: Sample + cpal::SizedSample,
    <T as Sample>::Float: Into<f64>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            for frame in data.chunks(channels) {
                if channels >= 1 {
                    let s = frame[0].to_float_sample().into() as f32;
                    let _ = prod_l.push(s);
                }
                if channels >= 2 {
                    let s = frame[1].to_float_sample().into() as f32;
                    let _ = prod_r.push(s);
                } else {
                    let s = frame[0].to_float_sample().into() as f32;
                    let _ = prod_r.push(s);
                }
            }
        },
        err_fn,
        None,
    );

    match stream {
        Ok(s) => Ok(s),
        Err(e) => {
            logger.try_run::<()>(Err(StringError(format!(
                "Failed to build input stream: {}",
                e
            ))
            .into()));
            Err(())
        }
    }
}

struct AudioApp<'a> {
    // Changed to UClassList<1>
    animation: KramaFrame<UClassList<1>, UFrameList<'a, 1, u8, TRES16Bits, i16>>,

    cons_l: Consumer<f32, Arc<ringbuf::HeapRb<f32>>>,
    cons_r: Consumer<f32, Arc<ringbuf::HeapRb<f32>>>,

    band_levels_l: [f32; NUM_BANDS],
    band_levels_r: [f32; NUM_BANDS],
    band_starts_l: [f32; NUM_BANDS],
    band_starts_r: [f32; NUM_BANDS],
    band_targets_l: [f32; NUM_BANDS],
    band_targets_r: [f32; NUM_BANDS],

    // Sampling Average / Temporal Smoothing Buffers
    spectrum_smooth_l: [f32; NUM_BANDS],
    spectrum_smooth_r: [f32; NUM_BANDS],

    planner: FftPlanner<f32>,
    fft_input_l: Vec<Complex<f32>>,
    fft_input_r: Vec<Complex<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    window_table: Vec<f32>,

    raw_buf_l: Vec<f32>,
    raw_buf_r: Vec<f32>,

    anim_duration_ms: u16,
    current_easing_index: usize,
    quit: bool,

    // Radiance Effect
    bass_energy: f32,
    radiance_phase: f32,

    // Status
    show_status: bool,
    device_name: String,
}

impl<'a> AudioApp<'a> {
    fn new(
        animation: KramaFrame<UClassList<1>, UFrameList<'a, 1, u8, TRES16Bits, i16>>,
        cons_l: Consumer<f32, Arc<ringbuf::HeapRb<f32>>>,
        cons_r: Consumer<f32, Arc<ringbuf::HeapRb<f32>>>,
        device_name: String,
    ) -> Self {
        let mut window_table = Vec::with_capacity(FFT_SIZE);
        for i in 0..FFT_SIZE {
            let n = i as f32;
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n / (FFT_SIZE as f32 - 1.0)).cos());
            window_table.push(w);
        }

        Self {
            animation,
            cons_l,
            cons_r,
            band_levels_l: [0.0; NUM_BANDS],
            band_levels_r: [0.0; NUM_BANDS],
            band_starts_l: [0.0; NUM_BANDS],
            band_starts_r: [0.0; NUM_BANDS],
            band_targets_l: [0.0; NUM_BANDS],
            band_targets_r: [0.0; NUM_BANDS],

            spectrum_smooth_l: [0.0; NUM_BANDS],
            spectrum_smooth_r: [0.0; NUM_BANDS],

            planner: FftPlanner::new(),
            fft_input_l: vec![Complex::zero(); FFT_SIZE],
            fft_input_r: vec![Complex::zero(); FFT_SIZE],
            fft_scratch: vec![Complex::zero(); FFT_SIZE],
            window_table,

            raw_buf_l: vec![0.0; FFT_SIZE],
            raw_buf_r: vec![0.0; FFT_SIZE],

            anim_duration_ms: 140, // Default requested
            current_easing_index: 3,
            quit: false,

            bass_energy: 0.0,
            radiance_phase: 0.0,

            show_status: false,
            device_name,
        }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        logger: &mut MultiLogger<Instant, ErrorConv>,
    ) {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);

        let fft = self.planner.plan_fft_forward(FFT_SIZE);

        while !self.quit {
            let res = terminal.draw(|f| self.view(f));
            logger.try_run(res.map_err(|e| StringError(format!("Draw error: {}", e)).into()));

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));
            if event::poll(timeout).unwrap_or(false) {
                if let Ok(event::Event::Key(key)) = event::read() {
                    match key.code {
                        KeyCode::Char('q') => self.quit = true,
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            self.show_status = !self.show_status;
                        }
                        KeyCode::Up => {
                            self.anim_duration_ms =
                                self.anim_duration_ms.saturating_add(5).min(800);
                            self.update_animation_settings();
                        }
                        KeyCode::Down => {
                            self.anim_duration_ms = self.anim_duration_ms.saturating_sub(5).max(1);
                            self.update_animation_settings();
                        }
                        KeyCode::Right => {
                            self.current_easing_index = (self.current_easing_index + 1) % 7;
                            self.update_animation_settings();
                        }
                        KeyCode::Left => {
                            self.current_easing_index = if self.current_easing_index == 0 {
                                6
                            } else {
                                self.current_easing_index - 1
                            };
                            self.update_animation_settings();
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                let popped_l = self.cons_l.pop_iter().collect::<Vec<f32>>();
                let popped_r = self.cons_r.pop_iter().collect::<Vec<f32>>();

                if !popped_l.is_empty() {
                    let len = self.raw_buf_l.len();
                    let pop_len = popped_l.len();
                    if pop_len >= len {
                        self.raw_buf_l.copy_from_slice(&popped_l[pop_len - len..]);
                    } else {
                        self.raw_buf_l.copy_within(pop_len.., 0);
                        self.raw_buf_l[len - pop_len..].copy_from_slice(&popped_l);
                    }
                }

                if !popped_r.is_empty() {
                    let len = self.raw_buf_r.len();
                    let pop_len = popped_r.len();
                    if pop_len >= len {
                        self.raw_buf_r.copy_from_slice(&popped_r[pop_len - len..]);
                    } else {
                        self.raw_buf_r.copy_within(pop_len.., 0);
                        self.raw_buf_r[len - pop_len..].copy_from_slice(&popped_r);
                    }
                }

                for (i, w) in self.window_table.iter().enumerate() {
                    self.fft_input_l[i] = Complex {
                        re: self.raw_buf_l[i] * w,
                        im: 0.0,
                    };
                    self.fft_input_r[i] = Complex {
                        re: self.raw_buf_r[i] * w,
                        im: 0.0,
                    };
                }

                fft.process_with_scratch(&mut self.fft_input_l, &mut self.fft_scratch);
                fft.process_with_scratch(&mut self.fft_input_r, &mut self.fft_scratch);

                let mut next_fft_l = [0.0f32; NUM_BANDS];
                let mut next_fft_r = [0.0f32; NUM_BANDS];
                process_fft_bands(&self.fft_input_l, &mut next_fft_l);
                process_fft_bands(&self.fft_input_r, &mut next_fft_r);

                self.animation.update_progress(TRES16Bits::from_millis(
                    last_tick.elapsed().as_millis() as u16,
                ));

                // Optimized Animation Logic: Single ID (0) drives global synchronization
                let p_global = self.animation.get_progress_f32("viz", 0);
                let mut update_targets = false;

                if p_global >= 1.0 || p_global == 0.0 {
                    self.animation.restart_progress("viz", 0);
                    update_targets = true;
                }

                // Apply continuous averaging (Temporal Smoothing)
                for i in 0..NUM_BANDS {
                    // Running average: 50% history, 50% new data
                    // This continuously averages the audio for each bar
                    self.spectrum_smooth_l[i] = (self.spectrum_smooth_l[i] + next_fft_l[i]) * 0.5;
                    self.spectrum_smooth_r[i] = (self.spectrum_smooth_r[i] + next_fft_r[i]) * 0.5;

                    // Update next_fft to use smoothed values
                    next_fft_l[i] = self.spectrum_smooth_l[i];
                    next_fft_r[i] = self.spectrum_smooth_r[i];
                }

                for i in 0..NUM_BANDS {
                    // Update animation targets only when the global cycle resets
                    if update_targets {
                        self.band_starts_l[i] = self.band_targets_l[i];
                        self.band_targets_l[i] = next_fft_l[i];

                        self.band_starts_r[i] = self.band_targets_r[i];
                        self.band_targets_r[i] = next_fft_r[i];
                    }

                    // Use global progress (ID 0) for all bands
                    self.band_levels_l[i] = self.animation.get_value_byrange_inclusive(
                        "viz",
                        0,
                        self.band_starts_l[i]..=self.band_targets_l[i],
                    );

                    self.band_levels_r[i] = self.animation.get_value_byrange_inclusive(
                        "viz",
                        0,
                        self.band_starts_r[i]..=self.band_targets_r[i],
                    );
                }

                // --- Radiance Logic ---
                // Calculate bass energy from the first few bands (approx 0-250Hz)
                // Since bands are 60 now, take first 8 to cover similar range
                let bass_sum_l: f32 = self.band_levels_l.iter().take(8).sum();
                let bass_sum_r: f32 = self.band_levels_r.iter().take(8).sum();
                let instant_bass = (bass_sum_l + bass_sum_r) / 16.0; // Normalized roughly 0.0-1.0

                // Smooth energy follower
                self.bass_energy = self.bass_energy * 0.90 + instant_bass * 0.10;

                // Animate phase based on bass energy for sync
                // Base speed 0.02 + dynamic component
                let phase_step = 0.02 + (self.bass_energy * 0.20);
                self.radiance_phase =
                    (self.radiance_phase + phase_step) % (std::f32::consts::PI * 2.0);

                last_tick = Instant::now();
            }
        }
    }

    fn update_animation_settings(&mut self) {
        let t = TRES16Bits::from_millis(self.anim_duration_ms);
        let easing = match self.current_easing_index {
            0 => kramaframe::keyframe::KeyFrameFunction::Linear,
            1 => kramaframe::keyframe::KeyFrameFunction::Ease,
            2 => kramaframe::keyframe::KeyFrameFunction::EaseIn,
            3 => kramaframe::keyframe::KeyFrameFunction::EaseOut,
            4 => kramaframe::keyframe::KeyFrameFunction::EaseInOut,
            5 => kramaframe::keyframe::KeyFrameFunction::Quadratic,
            6 => kramaframe::keyframe::KeyFrameFunction::new_cubic_bezier_f32(0.3, 0.0, 0.8, 0.15),
            _ => kramaframe::keyframe::KeyFrameFunction::EaseOut,
        };
        for (_, func) in self.animation.classlist.0.iter_mut() {
            *func = easing;
        }
        // Update only ID 0
        self.animation.set_timing("viz", 0, t.clone());
    }

    fn view(&mut self, f: &mut Frame) {
        // Full screen visualization, no text widgets
        self.render_merged_viz(f, f.area());

        if self.show_status {
            self.render_status(f);
        }
    }

    fn render_status(&self, f: &mut Frame) {
        let easing_name = match self.current_easing_index {
            0 => "Linear",
            1 => "Ease",
            2 => "EaseIn",
            3 => "EaseOut",
            4 => "EaseInOut",
            5 => "Quadratic",
            6 => "Cubic Bezier",
            _ => "Unknown",
        };

        let mut status_text = vec![
            Line::from(vec![
                Span::styled("Device: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.device_name, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Easing: ", Style::default().fg(Color::Gray)),
                Span::styled(easing_name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}ms", self.anim_duration_ms),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        status_text.push(Line::from(""));
        status_text.push(Line::from(Span::styled(
            "Press 's' to hide",
            Style::default().fg(Color::DarkGray),
        )));

        let status = Paragraph::new(status_text).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Status "),
        );

        let area = f.area();
        let status_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: 40.min(area.width.saturating_sub(2)),
            height: 6.min(area.height.saturating_sub(2)),
        };

        f.render_widget(status, status_area);
    }

    fn render_merged_viz(&self, f: &mut Frame, area: Rect) {
        let inner = area;
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let gap_width: u16 = 1;
        let max_total_bands =
            (inner.width.saturating_add(gap_width)) as usize / (1 + gap_width as usize);
        let num_per_side = (max_total_bands / 2).min(NUM_BANDS).max(1);
        let total_bands = num_per_side * 2;

        let total_gap = (total_bands.saturating_sub(1) as u16) * gap_width;
        let available = inner.width.saturating_sub(total_gap);
        let band_width = (available as usize / total_bands).max(1) as u16;

        let total_w = (total_bands as u16 * band_width) + total_gap;
        let start_x = inner.x + (inner.width.saturating_sub(total_w)) / 2;

        // --- Color Gradient Settings ---

        // Base Theme Colors (Edge Color)
        let (end_r, end_g, end_b) = match self.current_easing_index {
            0 => (0.0, 255.0, 255.0),   // Linear: Cyan
            1 => (255.0, 65.0, 54.0),   // Ease: Red (Aurora Red)
            2 => (147.0, 112.0, 219.0), // EaseIn: MediumPurple
            3 => (255.0, 165.0, 0.0),   // EaseOut: Orange
            4 => (255.0, 105.0, 180.0), // EaseInOut: HotPink
            5 => (80.0, 200.0, 120.0),  // Quadratic: Emerald Green
            6 => (255.0, 215.0, 0.0),   // Gravity: Gold
            _ => (94.0, 129.0, 172.0),  // Fallback: Nord Blue
        };

        // Radiance Calculation
        let radiance = (self.bass_energy * 2.5).clamp(0.0, 1.0).powf(1.5);

        // Logic:
        // Low Bass: Center color matches Edge color (Solid look, no white)
        // High Bass: Center color becomes White and spreads outwards

        let mut start_r = end_r;
        let mut start_g = end_g;
        let mut start_b = end_b;

        // Only add white center if radiance is significant (insane bass)
        if radiance > 0.15 {
            let target_r = 255.0;
            let target_g = 255.0;
            let target_b = 255.0;

            // Blend to white based on radiance intensity
            let blend = ((radiance - 0.15) / 0.85).clamp(0.0, 1.0);

            start_r = start_r + (target_r - start_r) * blend;
            start_g = start_g + (target_g - start_g) * blend;
            start_b = start_b + (target_b - start_b) * blend;
        }

        // Spread Factor:
        // Higher power = remains close to 0 (Center/Start Color) longer.
        // This makes the "White" (if active) spread out from the center.
        // User requested multiplier to be 0.5
        let spread_power = 1.0 + (radiance * 0.5);

        for j in 0..total_bands {
            let (idx_in_side, levels, _) = if j < num_per_side {
                (num_per_side - 1 - j, &self.band_levels_l, true)
            } else {
                (j - num_per_side, &self.band_levels_r, false)
            };

            let source_idx =
                (idx_in_side * (NUM_BANDS - 1) / num_per_side.max(1)).min(NUM_BANDS - 1);
            let val = levels[source_idx];
            let ratio = source_idx as f32 / (NUM_BANDS - 1) as f32;

            // Interpolate Color: Center -> Edge
            // warp goes from 0 (Center) to 1 (Edge)
            let warp = ratio.powf(spread_power);

            let r = (start_r + (end_r - start_r) * warp) as u8;
            let g = (start_g + (end_g - start_g) * warp) as u8;
            let b = (start_b + (end_b - start_b) * warp) as u8;
            let color = Color::Rgb(r, g, b);

            let x = start_x + j as u16 * (band_width + gap_width);
            let h = val * inner.height as f32;
            let h_int = h.floor() as u16;

            if h_int > 0 && h_int <= inner.height {
                let y = inner.y + inner.height - h_int;
                if x < inner.x + inner.width {
                    f.buffer_mut().set_style(
                        Rect {
                            x,
                            y,
                            width: band_width.min(inner.width - (x - inner.x)),
                            height: h_int,
                        },
                        Style::default().bg(color),
                    );
                }
            }

            let frac = h - h_int as f32;
            if frac > 0.01 && h_int < inner.height {
                let y = inner.y + inner.height - 1 - h_int;
                let symbol = match frac {
                    v if v > 0.875 => "█",
                    v if v > 0.75 => "▇",
                    v if v > 0.625 => "▆",
                    v if v > 0.5 => "▅",
                    v if v > 0.375 => "▄",
                    v if v > 0.25 => "▃",
                    v if v > 0.125 => "▂",
                    _ => "▁",
                };
                if x < inner.x + inner.width {
                    let line = Line::raw(symbol.repeat(band_width as usize)).fg(color);
                    f.render_widget(
                        Paragraph::new(line),
                        Rect {
                            x,
                            y,
                            width: band_width,
                            height: 1,
                        },
                    );
                }
            }
        }
    }
}

fn process_fft_bands(fft_output: &[Complex<f32>], bands: &mut [f32]) {
    // Logarithmic mapping constants for smoother grouping
    let num_bins = fft_output.len() / 2; // ~1024
    let log_min = 1.0f32.ln(); // Start at bin 1
    let log_max = (num_bins as f32).ln();

    for i in 0..NUM_BANDS {
        // Calculate logarithmic range for this band
        let log_start = log_min + (log_max - log_min) * (i as f32 / NUM_BANDS as f32);
        let log_end = log_min + (log_max - log_min) * ((i + 1) as f32 / NUM_BANDS as f32);

        let mut start_bin = log_start.exp() as usize;
        let mut end_bin = log_end.exp() as usize;

        // Ensure strictly monotonic and bounds
        start_bin = start_bin.max(1).min(num_bins - 1);
        end_bin = end_bin.max(start_bin + 1).min(num_bins);

        let mut sum_amp = 0.0f32;
        let count = (end_bin - start_bin) as f32;

        for bin_idx in start_bin..end_bin {
            if bin_idx < fft_output.len() {
                // Sum of magnitudes
                sum_amp += fft_output[bin_idx].norm();
            }
        }

        // Average the magnitude ("average bar")
        let avg_amp = if count > 0.0 { sum_amp / count } else { 0.0 };

        // Apply slight frequency skew boost for higher frequencies
        let freq_skew = 1.0 + (i as f32 / NUM_BANDS as f32) * 2.5;

        let display_val = (avg_amp * freq_skew / 100.0).clamp(0.0, 1.0);
        bands[i] = display_val;
    }
}

#[derive(Clone)]
pub struct ErrorConv;

impl StorageProvider for ErrorConv {
    fn write_data(&mut self, args: std::fmt::Arguments, debuglevel: &StatusLevel) {
        if matches!(debuglevel, StatusLevel::Error | StatusLevel::Critical) {
            ratatui::restore();
            let _ = disable_raw_mode().is_ok();
        }
        println!("{}", args)
    }
}
