// Timers
// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Timers
//!
//! The schedule behind `Engine::on_timer`.  It only decides *when* timers are due, from
//! [`Instant`]s the caller passes in; the callbacks live in the `Engine`.  Keeping the clock out of
//! the schedule makes it testable without waiting.

use std::time::{Duration, Instant};

/// A handle identifying one timer, returned by `Engine::on_timer`.
///
/// IDs are unique within an `Engine` and are never reused.  Pass one to `Ui::cancel_timer` or
/// `Engine::cancel_timer` to stop the timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimerId(pub u64);

/// One scheduled timer.
struct Entry {
    id: TimerId,
    interval: Duration,
    repeat: bool,
    due: Instant,
}

/// The timers that are waiting to fire.
#[derive(Default)]
pub(crate) struct TimerSchedule {
    next_id: u64,
    entries: Vec<Entry>,
}

impl TimerSchedule {
    /// Schedules a timer to first fire one `interval` after `now`.
    ///
    /// # Arguments
    ///
    /// * `now` - The current time.
    /// * `interval` - The time until the timer fires, and between firings if it repeats.
    /// * `repeat` - `true` to fire every `interval`; `false` to fire once.
    ///
    /// # Returns
    ///
    /// The new timer's ID.
    pub(crate) fn add(&mut self, now: Instant, interval: Duration, repeat: bool) -> TimerId {
        let id = TimerId(self.next_id);

        self.next_id += 1;
        self.entries.push(Entry {
            id,
            interval,
            repeat,
            due: now + interval,
        });

        id
    }

    /// Stops a timer.
    ///
    /// # Arguments
    ///
    /// * `id` - The timer to stop.
    ///
    /// # Returns
    ///
    /// `true` if the timer was waiting; `false` if it had already fired (one-shot) or been
    /// cancelled.
    pub(crate) fn cancel(&mut self, id: TimerId) -> bool {
        let before = self.entries.len();

        self.entries.retain(|entry| entry.id != id);

        self.entries.len() != before
    }

    /// Returns whether a timer is still waiting.
    ///
    /// # Arguments
    ///
    /// * `id` - The timer to look for.
    ///
    /// # Returns
    ///
    /// `true` if the timer is scheduled.
    pub(crate) fn contains(&self, id: TimerId) -> bool {
        self.entries.iter().any(|entry| entry.id == id)
    }

    /// Returns the timers due at `now`, earliest first, without changing the schedule.
    ///
    /// # Arguments
    ///
    /// * `now` - The current time.
    ///
    /// # Returns
    ///
    /// The IDs of the timers whose time has come; call [`TimerSchedule::complete`] for each.
    pub(crate) fn due(&self, now: Instant) -> Vec<TimerId> {
        let mut due: Vec<&Entry> = self.entries.iter().filter(|e| e.due <= now).collect();

        due.sort_by_key(|entry| (entry.due, entry.id));
        due.into_iter().map(|entry| entry.id).collect()
    }

    /// Marks a due timer as fired: a one-shot is removed, and a repeating timer moves to its next
    /// time.  A repeating timer that fell several intervals behind fires once, then keeps its
    /// rhythm, instead of firing in a burst.
    ///
    /// # Arguments
    ///
    /// * `id` - The timer that is about to fire.
    /// * `now` - The current time.
    ///
    /// # Returns
    ///
    /// `Some(repeat)` if the timer fires (`repeat` is whether it stays scheduled), or `None` if it
    /// was cancelled since [`TimerSchedule::due`] was called.
    pub(crate) fn complete(&mut self, id: TimerId, now: Instant) -> Option<bool> {
        let index = self.entries.iter().position(|entry| entry.id == id)?;
        let entry = &mut self.entries[index];

        if !entry.repeat {
            self.entries.remove(index);

            return Some(false);
        }

        // A zero interval would never get ahead of `now`; fire it once per call.
        if entry.interval.is_zero() {
            entry.due = now;
        } else {
            while entry.due <= now {
                entry.due += entry.interval;
            }
        }

        Some(true)
    }

    /// Returns the time until the next timer is due.
    ///
    /// # Arguments
    ///
    /// * `now` - The current time.
    ///
    /// # Returns
    ///
    /// The wait (zero if one is already due), or `None` if no timers are scheduled.
    pub(crate) fn next_wait(&self, now: Instant) -> Option<Duration> {
        self.entries
            .iter()
            .map(|entry| entry.due.saturating_duration_since(now))
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_timer_is_due_only_after_its_interval() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();
        let id = s.add(t0, ms(100), false);

        assert!(s.due(t0 + ms(99)).is_empty());
        assert_eq!(s.due(t0 + ms(100)), vec![id]);
    }

    #[test]
    fn a_one_shot_fires_once() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();
        let id = s.add(t0, ms(10), false);

        assert_eq!(s.complete(id, t0 + ms(10)), Some(false));
        assert!(!s.contains(id));
        assert!(s.due(t0 + ms(500)).is_empty());
    }

    #[test]
    fn a_repeating_timer_keeps_its_rhythm_and_skips_missed_intervals() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();
        let id = s.add(t0, ms(100), true);

        // Seen late, at 350 ms: one firing, next due at 400 ms.
        assert_eq!(s.complete(id, t0 + ms(350)), Some(true));
        assert!(s.due(t0 + ms(399)).is_empty());
        assert_eq!(s.due(t0 + ms(400)), vec![id]);
    }

    #[test]
    fn cancelling_removes_the_timer_once() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();
        let id = s.add(t0, ms(10), true);

        assert!(s.cancel(id));
        assert!(!s.cancel(id));
        assert_eq!(s.complete(id, t0 + ms(10)), None);
        assert!(!s.contains(id));
    }

    #[test]
    fn due_timers_are_ordered_and_ids_are_not_reused() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();
        let late = s.add(t0, ms(50), false);
        let early = s.add(t0, ms(10), false);

        assert_ne!(late, early);
        assert_eq!(s.due(t0 + ms(60)), vec![early, late]);

        s.cancel(late);

        assert_ne!(s.add(t0, ms(1), false), late);
    }

    #[test]
    fn next_wait_reports_the_nearest_timer() {
        let t0 = Instant::now();
        let mut s = TimerSchedule::default();

        assert_eq!(s.next_wait(t0), None);

        s.add(t0, ms(300), false);
        s.add(t0, ms(120), false);

        assert_eq!(s.next_wait(t0 + ms(20)), Some(ms(100)));
        assert_eq!(s.next_wait(t0 + ms(400)), Some(Duration::ZERO));
    }
}
