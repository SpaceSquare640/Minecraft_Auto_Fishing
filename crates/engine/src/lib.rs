//! Auto-fishing state machine.
//!
//! Pure logic: no OS calls and no real clock. The caller feeds events with a
//! timestamp (time since app start) and performs the returned actions, so the
//! whole behaviour can be tested with a fake clock and replayed from recordings.
#![forbid(unsafe_code)]

use std::time::Duration;

/// Timing rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Bites are ignored this long after a cast: the bobber is still flying, and a Java
    /// caption from the previous bite stays on screen for up to 3 s while it fades.
    pub settle: Duration,
    /// No bite for this long: reel in and cast again (also rescues bites too far away to hear).
    pub bite_timeout: Duration,
    /// Pause between reeling in and casting again.
    pub recast_delay: Duration,
    /// Wait after Start before the first click: the start hotkey may still be held down, and
    /// Bedrock misses a click sent at the same moment.
    pub start_delay: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settle: Duration::from_millis(3000),
            bite_timeout: Duration::from_secs(45),
            recast_delay: Duration::from_millis(600),
            start_delay: Duration::from_millis(500),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Stopped by the player.
    Idle,
    /// Just started; the first click comes after the start delay.
    Starting,
    /// Line just cast; waiting for it to settle.
    Casting,
    /// Waiting for a bite.
    Waiting,
    /// Just reeled in; waiting to cast again.
    Reeling,
    /// The game window lost focus. Needs an explicit Start to continue.
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Start,
    Stop,
    /// A detector reported a bite.
    Bite,
    /// The game showed that the bobber was just thrown (Java "Bobber is thrown").
    LineOut,
    /// The game showed that the bobber was just pulled in (Java "Bobber is retrieved").
    LineIn,
    FocusLost,
    FocusGained,
    /// Periodic clock tick.
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Press the use-item button once (casts or reels in the rod).
    RightClick,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub casts: u32,
    pub bites: u32,
    pub timeouts: u32,
    /// Times the game showed a different bobber state than expected.
    pub resyncs: u32,
}

#[derive(Debug)]
pub struct Engine {
    cfg: Config,
    state: State,
    since: Duration,
    focused: bool,
    line_out: bool,
    /// The game reported the bobber since the last Stop. Our own belief is not trusted after
    /// a Stop, because the player may use the rod by hand while we are stopped.
    line_reported: bool,
    stats: Stats,
}

impl Engine {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            state: State::Idle,
            since: Duration::ZERO,
            focused: false,
            line_out: false,
            line_reported: false,
            stats: Stats::default(),
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Whether the bobber is believed to be in the water.
    pub fn line_out(&self) -> bool {
        self.line_out
    }

    /// Feeds one event and returns the actions to perform, in order.
    pub fn step(&mut self, event: Event, now: Duration) -> Vec<Action> {
        let elapsed = now.saturating_sub(self.since);
        match (event, self.state) {
            (Event::FocusGained, _) => {
                // Never resume on our own: the player may be typing in another window.
                self.focused = true;
                vec![]
            }
            (Event::FocusLost, _) => {
                self.focused = false;
                if self.is_running() {
                    self.enter(State::Paused, now);
                }
                vec![]
            }
            // Stopping sends no input: the rod is left as it is.
            (Event::Stop, _) => {
                self.line_reported = false;
                self.enter(State::Idle, now);
                vec![]
            }
            // A fresh start assumes the line is in (Setup asks the player to reel in first),
            // unless the game told us otherwise. Resuming a pause keeps what we know.
            (Event::Start, State::Idle) => {
                if !self.line_reported {
                    self.line_out = false;
                }
                self.resume(now)
            }
            (Event::Start, State::Paused) => self.resume(now),
            // The game disagrees with our belief: follow the game. This repairs a click that
            // did the opposite of what we meant, e.g. after the game removed the bobber.
            (Event::LineIn, State::Casting | State::Waiting) => {
                self.line_out = false;
                self.stats.resyncs += 1;
                self.enter(State::Reeling, now);
                vec![]
            }
            (Event::LineOut, State::Reeling) => {
                self.line_out = true;
                self.stats.resyncs += 1;
                self.enter(State::Casting, now);
                vec![]
            }
            // While stopped or paused, just remember where the bobber is.
            (Event::LineIn, State::Idle | State::Paused | State::Starting) => {
                self.line_out = false;
                self.line_reported = true;
                vec![]
            }
            (Event::LineOut, State::Idle | State::Paused | State::Starting) => {
                self.line_out = true;
                self.line_reported = true;
                vec![]
            }
            (Event::Bite, State::Waiting) => {
                self.stats.bites += 1;
                self.reel(now)
            }
            (Event::Tick, State::Starting) if elapsed >= self.cfg.start_delay => {
                if self.line_out {
                    // The bobber is still out from before the pause: bring it in first.
                    self.reel(now)
                } else {
                    self.cast(now)
                }
            }
            (Event::Tick, State::Casting) if elapsed >= self.cfg.settle => {
                self.enter(State::Waiting, now);
                vec![]
            }
            (Event::Tick, State::Waiting) if elapsed >= self.cfg.bite_timeout => {
                self.stats.timeouts += 1;
                self.reel(now)
            }
            (Event::Tick, State::Reeling) if elapsed >= self.cfg.recast_delay => self.cast(now),
            // Everything else is ignored, e.g. a bite while casting or reeling.
            _ => vec![],
        }
    }

    /// The caller could not perform the last `RightClick` (e.g. the game lost focus a moment
    /// before sending). Undo its effect on the bobber and pause, so the next Start repeats it.
    pub fn input_failed(&mut self, now: Duration) {
        self.line_out = !self.line_out;
        if !self.line_out {
            // The failed click was a cast.
            self.stats.casts = self.stats.casts.saturating_sub(1);
        }
        self.enter(State::Paused, now);
    }

    fn is_running(&self) -> bool {
        matches!(
            self.state,
            State::Starting | State::Casting | State::Waiting | State::Reeling
        )
    }

    fn resume(&mut self, now: Duration) -> Vec<Action> {
        if !self.focused {
            self.enter(State::Paused, now);
            return vec![];
        }
        self.enter(State::Starting, now);
        vec![]
    }

    fn cast(&mut self, now: Duration) -> Vec<Action> {
        self.stats.casts += 1;
        self.line_out = true;
        self.enter(State::Casting, now);
        vec![Action::RightClick]
    }

    fn reel(&mut self, now: Duration) -> Vec<Action> {
        self.line_out = false;
        self.enter(State::Reeling, now);
        vec![Action::RightClick]
    }

    fn enter(&mut self, state: State, now: Duration) {
        self.state = state;
        self.since = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLICK: [Action; 1] = [Action::RightClick];

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    fn cfg() -> Config {
        Config {
            settle: ms(1000),
            bite_timeout: ms(10_000),
            recast_delay: ms(500),
            // Zero here so the other tests can start and cast at the same instant.
            start_delay: Duration::ZERO,
        }
    }

    /// Start, then the tick that ends the start delay; returns that tick's actions.
    fn start(e: &mut Engine, at: Duration) -> Vec<Action> {
        assert!(e.step(Event::Start, at).is_empty());
        e.step(Event::Tick, at)
    }

    /// Focused engine that has just cast at t = 0.
    fn casting() -> Engine {
        let mut e = Engine::new(cfg());
        e.step(Event::FocusGained, ms(0));
        assert_eq!(start(&mut e, ms(0)), CLICK);
        e
    }

    fn waiting() -> Engine {
        let mut e = casting();
        assert!(e.step(Event::Tick, ms(1000)).is_empty());
        assert_eq!(e.state(), State::Waiting);
        e
    }

    #[test]
    fn start_casts_when_focused() {
        let e = casting();
        assert_eq!(e.state(), State::Casting);
        assert!(e.line_out());
        assert_eq!(e.stats().casts, 1);
    }

    #[test]
    fn start_without_focus_pauses_and_sends_nothing() {
        let mut e = Engine::new(cfg());
        assert!(e.step(Event::Start, ms(0)).is_empty());
        assert_eq!(e.state(), State::Paused);
    }

    #[test]
    fn bite_while_settling_is_ignored() {
        let mut e = casting();
        assert!(e.step(Event::Bite, ms(300)).is_empty());
        assert!(e.step(Event::Tick, ms(999)).is_empty());
        assert_eq!(e.state(), State::Casting);
        assert_eq!(e.stats().bites, 0);
    }

    #[test]
    fn full_cycle_bite_reel_recast() {
        let mut e = waiting();
        assert_eq!(e.step(Event::Bite, ms(4000)), CLICK);
        assert_eq!(e.state(), State::Reeling);
        assert!(!e.line_out());
        assert!(e.step(Event::Tick, ms(4400)).is_empty());
        assert_eq!(e.step(Event::Tick, ms(4500)), CLICK);
        assert_eq!(e.state(), State::Casting);
        assert_eq!(
            e.stats(),
            Stats {
                casts: 2,
                bites: 1,
                timeouts: 0,
                resyncs: 0
            }
        );
    }

    #[test]
    fn bite_while_reeling_is_ignored() {
        let mut e = waiting();
        e.step(Event::Bite, ms(2000));
        assert!(e.step(Event::Bite, ms(2100)).is_empty());
        assert_eq!(e.stats().bites, 1);
    }

    #[test]
    fn timeout_reels_in_then_recasts() {
        let mut e = waiting();
        assert!(e.step(Event::Tick, ms(10_999)).is_empty());
        assert_eq!(e.step(Event::Tick, ms(11_000)), CLICK);
        assert_eq!(e.state(), State::Reeling);
        assert_eq!(e.step(Event::Tick, ms(11_500)), CLICK);
        assert_eq!(
            e.stats(),
            Stats {
                casts: 2,
                bites: 0,
                timeouts: 1,
                resyncs: 0
            }
        );
    }

    #[test]
    fn focus_loss_pauses_and_resume_reels_in_first() {
        let mut e = waiting();
        assert!(e.step(Event::FocusLost, ms(3000)).is_empty());
        assert_eq!(e.state(), State::Paused);
        // Ticks and bites do nothing while paused; regaining focus does not resume.
        assert!(e.step(Event::Tick, ms(60_000)).is_empty());
        assert!(e.step(Event::Bite, ms(60_000)).is_empty());
        assert!(e.step(Event::FocusGained, ms(61_000)).is_empty());
        assert_eq!(e.state(), State::Paused);
        // Start: the bobber is still out, so the first click reels it in.
        assert_eq!(start(&mut e, ms(62_000)), CLICK);
        assert_eq!(e.state(), State::Reeling);
        assert_eq!(e.step(Event::Tick, ms(62_500)), CLICK);
        assert_eq!(e.state(), State::Casting);
    }

    #[test]
    fn resume_without_focus_stays_paused() {
        let mut e = waiting();
        e.step(Event::FocusLost, ms(3000));
        assert!(e.step(Event::Start, ms(4000)).is_empty());
        assert_eq!(e.state(), State::Paused);
    }

    #[test]
    fn stop_sends_nothing_and_freezes() {
        let mut e = waiting();
        assert!(e.step(Event::Stop, ms(2000)).is_empty());
        assert_eq!(e.state(), State::Idle);
        assert!(e.step(Event::Tick, ms(60_000)).is_empty());
        assert!(e.step(Event::Bite, ms(60_000)).is_empty());
    }

    #[test]
    fn start_while_running_is_ignored() {
        let mut e = waiting();
        assert!(e.step(Event::Start, ms(2000)).is_empty());
        assert_eq!(e.state(), State::Waiting);
        assert_eq!(e.stats().casts, 1);
    }

    #[test]
    fn failed_cast_is_undone_and_repeated_on_start() {
        let mut e = casting();
        e.input_failed(ms(10));
        assert_eq!(e.state(), State::Paused);
        assert!(!e.line_out());
        assert_eq!(e.stats().casts, 0);
        assert_eq!(start(&mut e, ms(1000)), CLICK);
        assert_eq!(e.state(), State::Casting);
        assert!(e.line_out());
    }

    #[test]
    fn failed_reel_is_undone_and_repeated_on_start() {
        let mut e = waiting();
        e.step(Event::Bite, ms(2000));
        e.input_failed(ms(2001));
        assert!(e.line_out());
        assert_eq!(start(&mut e, ms(3000)), CLICK);
        assert_eq!(e.state(), State::Reeling);
        assert!(!e.line_out());
    }

    #[test]
    fn bobber_removed_by_the_game_is_repaired_at_the_timeout() {
        // The game silently removed the bobber; we still believe it is out.
        let mut e = waiting();
        assert_eq!(e.step(Event::Tick, ms(11_000)), CLICK); // "reel" - really a cast
        assert_eq!(e.state(), State::Reeling);
        assert!(e.step(Event::LineOut, ms(11_200)).is_empty()); // "Bobber is thrown"
        assert_eq!(e.state(), State::Casting);
        assert!(e.line_out());
        assert!(e.step(Event::Tick, ms(11_800)).is_empty()); // no recast click
        assert!(e.step(Event::Tick, ms(14_200)).is_empty());
        assert_eq!(e.state(), State::Waiting);
        assert_eq!(e.stats().resyncs, 1);
    }

    #[test]
    fn cast_that_pulled_the_bobber_in_is_repeated() {
        let mut e = casting();
        assert!(e.step(Event::LineIn, ms(200)).is_empty()); // "Bobber is retrieved"
        assert_eq!(e.state(), State::Reeling);
        assert!(!e.line_out());
        assert_eq!(e.step(Event::Tick, ms(800)), CLICK); // cast again
        assert_eq!(e.state(), State::Casting);
    }

    #[test]
    fn expected_bobber_events_change_nothing() {
        let mut e = casting();
        assert!(e.step(Event::LineOut, ms(200)).is_empty());
        assert_eq!(e.state(), State::Casting);
        e.step(Event::Tick, ms(1000));
        e.step(Event::Bite, ms(2000));
        assert!(e.step(Event::LineIn, ms(2100)).is_empty());
        assert_eq!(e.state(), State::Reeling);
        assert_eq!(e.stats().resyncs, 0);
    }

    #[test]
    fn manual_cast_while_paused_is_remembered() {
        let mut e = Engine::new(cfg());
        e.step(Event::FocusGained, ms(0));
        e.step(Event::LineOut, ms(100)); // player cast by hand before starting
        assert_eq!(start(&mut e, ms(200)), CLICK);
        assert_eq!(e.state(), State::Reeling); // so the first click reels in
    }

    #[test]
    fn start_after_stop_casts_first() {
        // Stopped with the line out; the player reels in by hand, which we cannot see.
        let mut e = waiting();
        e.step(Event::Stop, ms(2000));
        assert_eq!(start(&mut e, ms(5000)), CLICK);
        assert_eq!(e.state(), State::Casting);
        assert_eq!(e.stats().casts, 2);
    }

    #[test]
    fn resume_after_pause_still_reels_first() {
        let mut e = waiting();
        e.step(Event::FocusLost, ms(2000));
        e.step(Event::FocusGained, ms(3000));
        assert_eq!(start(&mut e, ms(4000)), CLICK);
        assert_eq!(e.state(), State::Reeling);
    }

    fn delayed() -> Engine {
        let mut e = Engine::new(Config {
            start_delay: ms(500),
            ..cfg()
        });
        e.step(Event::FocusGained, ms(0));
        e
    }

    #[test]
    fn first_click_waits_for_the_start_delay() {
        let mut e = delayed();
        assert!(e.step(Event::Start, ms(1000)).is_empty());
        assert_eq!(e.state(), State::Starting);
        assert!(e.step(Event::Bite, ms(1200)).is_empty());
        assert!(e.step(Event::Tick, ms(1499)).is_empty());
        assert_eq!(e.step(Event::Tick, ms(1500)), CLICK);
        assert_eq!(e.state(), State::Casting);
        assert_eq!(e.stats().casts, 1);
    }

    #[test]
    fn stop_during_the_start_delay_cancels() {
        let mut e = delayed();
        e.step(Event::Start, ms(1000));
        assert!(e.step(Event::Stop, ms(1200)).is_empty());
        assert!(e.step(Event::Tick, ms(2000)).is_empty());
        assert_eq!(e.state(), State::Idle);
        assert_eq!(e.stats().casts, 0);
    }

    #[test]
    fn focus_loss_during_the_start_delay_pauses() {
        let mut e = delayed();
        e.step(Event::Start, ms(1000));
        assert!(e.step(Event::FocusLost, ms(1200)).is_empty());
        assert!(e.step(Event::Tick, ms(2000)).is_empty());
        assert_eq!(e.state(), State::Paused);
        assert_eq!(e.stats().casts, 0);
    }

    #[test]
    fn clock_going_backwards_does_not_panic() {
        let mut e = waiting();
        assert!(e.step(Event::Tick, ms(500)).is_empty());
        assert_eq!(e.state(), State::Waiting);
    }
}
