use std::time::Duration as StdDuration;
use std::thread::{self, JoinHandle};
use std::collections::HashMap;

use chrono::{Duration, DateTime, Utc};
use parking_lot::ReentrantMutexGuard;

use crate::common::util;

struct JobState {
    action_factory: Box<dyn FnMut() -> Box<dyn FnMut() + Send + 'static> + Send + 'static>,
    last_execution_timestamp: DateTime<Utc>,
    trigger_duration: Option<Duration>,
}

pub struct State {
    jobs: HashMap<String, JobState>,
    terminated: bool,
}

impl State {
    pub fn new() -> State {
        State {
            jobs: HashMap::new(),
            terminated: false,
        }
    }
}

#[derive(Clone)]
pub struct Scheduler {
    daemon: super::Daemon,
}

impl Scheduler {
    pub fn new(daemon: super::Daemon) -> Scheduler {
        Scheduler { daemon, }
    }

    fn state(&self) -> ReentrantMutexGuard<super::State> {
        self.daemon.state.lock()
    }

    pub fn prepare<AF, A>(&self, job_name: &str, mut action_factory: AF)
        where AF: FnMut() -> A + Send + 'static,
              A: FnMut() + Send + 'static
    {
        let state = self.state();
        let mut g = state.scheduler.borrow_mut();
        
        debug!("Preparing job '{}' for scheduling", job_name);
        g.jobs.insert(job_name.into(), JobState {
            action_factory: Box::new(move || Box::new(action_factory())),
            last_execution_timestamp: util::past_timestamp(),
            trigger_duration: None,
        });
    }

    pub fn schedule(&self, job_name: &str, duration: Duration) {
        let state = self.state();
        let mut g = state.scheduler.borrow_mut();

        if let Some(ref mut js) = g.jobs.get_mut(job_name) {
            debug!("Scheduling job '{}' to be performed every {}", job_name, duration);
            js.trigger_duration = Some(duration);
        } else {
            warn!("Job '{}' does not exist, cannot schedule it", job_name);
        }
    }

    pub fn simulate(&self, job_name: &str) {
        let state = self.state();
        let mut g = state.scheduler.borrow_mut();

        trace!("Resetting last execution time of scheduled job '{}' to now", job_name);
        if let Some(ref mut js) = g.jobs.get_mut(job_name) {
            js.last_execution_timestamp = Utc::now();
        } else {
            warn!("Job '{}' does not exist, cannot simulate it", job_name);
        }
    }

    pub fn get_last_execution_timestamp(&self, job_name: &str) -> Option<DateTime<Utc>> {
        let state = self.state();
        let g = state.scheduler.borrow();

        if let Some(ref js) = g.jobs.get(job_name) {
            trace!("Retrieving the last execution time of job '{}'", job_name);
            Some(js.last_execution_timestamp)
        } else {
            warn!("Job '{}' does not exist, cannot get its last execution time", job_name);
            None
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        info!("Starting scheduler thread");
        let state = self.daemon.state.clone();
        thread::spawn(move || {
            let mut actions = Vec::new();
            loop {
                thread::sleep(StdDuration::from_secs(1));

                {
                    let state = state.lock();
                    let mut g = state.scheduler.borrow_mut();

                    if g.terminated { break; }

                    let current_timestamp = Utc::now();
                    for (job_name, job_state) in &mut g.jobs {
                        if let Some(trigger_duration) = job_state.trigger_duration {
                            let diff = current_timestamp.signed_duration_since(job_state.last_execution_timestamp);
                            if diff >= trigger_duration {
                                debug!("Executing scheduled job '{}'", job_name);
                                job_state.last_execution_timestamp = current_timestamp;
                                actions.push((job_state.action_factory)())
                            }
                        }
                    }
                }

                // execute actions outside of the scheduler mutex scope to avoid borrow panics
                for mut action in actions.drain(..) {
                    action();
                }
            }
        })
    }

    pub fn stop(&self) {
        info!("Stopping scheduler thread");
        let state = self.state();
        state.scheduler.borrow_mut().terminated = true;
    }
}
