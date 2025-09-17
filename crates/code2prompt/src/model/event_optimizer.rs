//! Event optimization system for TUI performance.
//!
//! This module provides event coalescing, throttling, and batching mechanisms
//! to prevent event queue buildup and improve TUI responsiveness when keys
//! are held down or rapid input occurs.

use crate::model::Message;
use ratatui::crossterm::event::KeyEvent;
use std::time::{Duration, Instant};

/// Event coalescer that merges similar events to prevent queue buildup
#[derive(Debug, Clone)]
pub struct EventCoalescer {
    /// Accumulated scroll delta
    pending_scroll: i32,
    /// Accumulated settings cursor movement
    pending_settings_move: i32,
    /// Accumulated statistics scroll
    pending_stats_scroll: i16,
    /// Accumulated output scroll
    pending_output_scroll: i16,
    /// Accumulated template picker movement
    pending_template_picker_move: i32,
    /// Last event time for flush timing
    last_event_time: Instant,
    /// Minimum time between flushes (for 60 FPS)
    flush_interval: Duration,
}

impl Default for EventCoalescer {
    fn default() -> Self {
        Self {
            pending_scroll: 0,
            pending_settings_move: 0,
            pending_stats_scroll: 0,
            pending_output_scroll: 0,
            pending_template_picker_move: 0,
            last_event_time: Instant::now(),
            flush_interval: Duration::from_millis(16), // 60 FPS
        }
    }
}

impl EventCoalescer {
    /// Add a scroll event to the coalescer
    pub fn add_scroll_event(&mut self, delta: i32) {
        self.pending_scroll += delta;
        self.last_event_time = Instant::now();
    }

    /// Add a settings cursor movement event
    pub fn add_settings_move(&mut self, delta: i32) {
        self.pending_settings_move += delta;
        self.last_event_time = Instant::now();
    }

    /// Add a statistics scroll event
    pub fn add_stats_scroll(&mut self, delta: i16) {
        self.pending_stats_scroll += delta;
        self.last_event_time = Instant::now();
    }

    /// Add an output scroll event
    pub fn add_output_scroll(&mut self, delta: i16) {
        self.pending_output_scroll += delta;
        self.last_event_time = Instant::now();
    }

    /// Add a template picker movement event
    pub fn add_template_picker_move(&mut self, delta: i32) {
        self.pending_template_picker_move += delta;
        self.last_event_time = Instant::now();
    }

    /// Check if events should be flushed
    pub fn should_flush(&self) -> bool {
        self.last_event_time.elapsed() >= self.flush_interval
    }

    /// Flush all pending events and return them as messages
    pub fn flush(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();

        // Flush scroll events
        if self.pending_scroll != 0 {
            messages.push(Message::MoveTreeCursor(self.pending_scroll));
            self.pending_scroll = 0;
        }

        // Flush settings movement
        if self.pending_settings_move != 0 {
            messages.push(Message::MoveSettingsCursor(self.pending_settings_move));
            self.pending_settings_move = 0;
        }

        // Flush statistics scroll
        if self.pending_stats_scroll != 0 {
            messages.push(Message::ScrollStatistics(self.pending_stats_scroll));
            self.pending_stats_scroll = 0;
        }

        // Flush output scroll
        if self.pending_output_scroll != 0 {
            messages.push(Message::ScrollOutput(self.pending_output_scroll));
            self.pending_output_scroll = 0;
        }

        // Flush template picker movement
        if self.pending_template_picker_move != 0 {
            messages.push(Message::TemplatePickerMove(
                self.pending_template_picker_move,
            ));
            self.pending_template_picker_move = 0;
        }

        messages
    }

    /// Check if there are any pending events
    pub fn has_pending_events(&self) -> bool {
        self.pending_scroll != 0
            || self.pending_settings_move != 0
            || self.pending_stats_scroll != 0
            || self.pending_output_scroll != 0
            || self.pending_template_picker_move != 0
    }
}

/// Render throttler to limit FPS and prevent unnecessary redraws
#[derive(Debug, Clone)]
pub struct RenderThrottler {
    last_render: Instant,
    target_fps: u32,
    frame_interval: Duration,
}

impl Default for RenderThrottler {
    fn default() -> Self {
        Self::new(60) // 60 FPS by default
    }
}

impl RenderThrottler {
    /// Create a new render throttler with specified FPS
    pub fn new(fps: u32) -> Self {
        let frame_interval = Duration::from_millis(1000 / fps.max(1) as u64);
        Self {
            last_render: Instant::now(),
            target_fps: fps,
            frame_interval,
        }
    }

    /// Check if a render should occur
    pub fn should_render(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_render) >= self.frame_interval {
            self.last_render = now;
            true
        } else {
            false
        }
    }

    /// Force a render (resets the timer)
    pub fn force_render(&mut self) {
        self.last_render = Instant::now();
    }

    /// Get current FPS
    pub fn get_fps(&self) -> u32 {
        self.target_fps
    }

    /// Set new FPS target
    pub fn set_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        self.frame_interval = Duration::from_millis(1000 / fps.max(1) as u64);
    }
}

/// Event queue drainer that processes all available events at once
#[derive(Debug, Default)]
pub struct EventQueueDrainer {
    /// Statistics for monitoring performance
    pub events_processed: u64,
    pub events_coalesced: u64,
    pub last_drain_time: Option<Instant>,
}

impl EventQueueDrainer {
    /// Drain all available events from the queue and coalesce them
    pub fn drain_events<F>(
        &mut self,
        mut event_handler: F,
        coalescer: &mut EventCoalescer,
    ) -> Vec<Message>
    where
        F: FnMut(KeyEvent) -> Option<Message>,
    {
        let mut messages = Vec::new();
        let mut events_this_drain = 0;

        // Process all available events without blocking
        while crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    events_this_drain += 1;

                    // Convert to ratatui KeyEvent
                    let ratatui_key = self.convert_key_event(key);

                    // Try to coalesce the event first
                    if let Some(message) = event_handler(ratatui_key) {
                        if self.try_coalesce_message(message.clone(), coalescer) {
                            self.events_coalesced += 1;
                        } else {
                            messages.push(message);
                        }
                    }
                }
            }
        }

        // Add any pending coalesced events
        messages.extend(coalescer.flush());

        self.events_processed += events_this_drain;
        self.last_drain_time = Some(Instant::now());

        messages
    }

    /// Try to coalesce a message into the coalescer
    fn try_coalesce_message(&self, message: Message, coalescer: &mut EventCoalescer) -> bool {
        match message {
            Message::MoveTreeCursor(delta) => {
                coalescer.add_scroll_event(delta);
                true
            }
            Message::MoveSettingsCursor(delta) => {
                coalescer.add_settings_move(delta);
                true
            }
            Message::ScrollStatistics(delta) => {
                coalescer.add_stats_scroll(delta);
                true
            }
            Message::ScrollOutput(delta) => {
                coalescer.add_output_scroll(delta);
                true
            }
            Message::TemplatePickerMove(delta) => {
                coalescer.add_template_picker_move(delta);
                true
            }
            _ => false, // Cannot coalesce this message
        }
    }

    /// Convert crossterm KeyEvent to ratatui KeyEvent
    fn convert_key_event(&self, key: crossterm::event::KeyEvent) -> KeyEvent {
        use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

        KeyEvent {
            code: match key.code {
                crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
                crossterm::event::KeyCode::Enter => KeyCode::Enter,
                crossterm::event::KeyCode::Left => KeyCode::Left,
                crossterm::event::KeyCode::Right => KeyCode::Right,
                crossterm::event::KeyCode::Up => KeyCode::Up,
                crossterm::event::KeyCode::Down => KeyCode::Down,
                crossterm::event::KeyCode::Home => KeyCode::Home,
                crossterm::event::KeyCode::End => KeyCode::End,
                crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
                crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
                crossterm::event::KeyCode::Tab => KeyCode::Tab,
                crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
                crossterm::event::KeyCode::Delete => KeyCode::Delete,
                crossterm::event::KeyCode::Insert => KeyCode::Insert,
                crossterm::event::KeyCode::F(n) => KeyCode::F(n),
                crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
                crossterm::event::KeyCode::Null => KeyCode::Null,
                crossterm::event::KeyCode::Esc => KeyCode::Esc,
                _ => KeyCode::Null, // Simplified for other key codes
            },
            modifiers: KeyModifiers::from_bits_truncate(key.modifiers.bits()),
            kind: match key.kind {
                crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
                crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat,
                crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
            },
            state: KeyEventState::from_bits_truncate(key.state.bits()),
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> EventDrainStats {
        EventDrainStats {
            events_processed: self.events_processed,
            events_coalesced: self.events_coalesced,
            coalesce_rate: if self.events_processed > 0 {
                self.events_coalesced as f64 / self.events_processed as f64
            } else {
                0.0
            },
            last_drain_time: self.last_drain_time,
        }
    }
}

/// Statistics for event draining performance
#[derive(Debug, Clone)]
pub struct EventDrainStats {
    pub events_processed: u64,
    pub events_coalesced: u64,
    pub coalesce_rate: f64,
    pub last_drain_time: Option<Instant>,
}

/// Complete event optimization system
#[derive(Debug)]
pub struct EventOptimizer {
    pub coalescer: EventCoalescer,
    pub render_throttler: RenderThrottler,
    pub queue_drainer: EventQueueDrainer,
}

impl Default for EventOptimizer {
    fn default() -> Self {
        Self {
            coalescer: EventCoalescer::default(),
            render_throttler: RenderThrottler::default(),
            queue_drainer: EventQueueDrainer::default(),
        }
    }
}

impl EventOptimizer {
    /// Create a new event optimizer with custom FPS
    pub fn new(fps: u32) -> Self {
        Self {
            coalescer: EventCoalescer::default(),
            render_throttler: RenderThrottler::new(fps),
            queue_drainer: EventQueueDrainer::default(),
        }
    }

    /// Process all events and return messages to handle
    pub fn process_events<F>(&mut self, event_handler: F) -> Vec<Message>
    where
        F: FnMut(KeyEvent) -> Option<Message>,
    {
        self.queue_drainer
            .drain_events(event_handler, &mut self.coalescer)
    }

    /// Check if rendering should occur
    pub fn should_render(&mut self) -> bool {
        self.render_throttler.should_render()
    }

    /// Force a render (useful for important state changes)
    pub fn force_render(&mut self) {
        self.render_throttler.force_render();
    }

    /// Get comprehensive performance statistics
    pub fn get_performance_stats(&self) -> EventOptimizerStats {
        EventOptimizerStats {
            drain_stats: self.queue_drainer.get_stats(),
            render_fps: self.render_throttler.get_fps(),
            has_pending_events: self.coalescer.has_pending_events(),
        }
    }
}

/// Complete performance statistics
#[derive(Debug, Clone)]
pub struct EventOptimizerStats {
    pub drain_stats: EventDrainStats,
    pub render_fps: u32,
    pub has_pending_events: bool,
}
