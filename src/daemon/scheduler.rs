use std::time::Duration as StdDuration;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::collections::HashMap;

use chrono::{Duration, DateTime, Utc};

use common::util;

#[derive(Clone)]
pub struct Scheduler {
    state: Arc<Mutex<State>>,
}

struct State {
    jobs: HashMap<String, JobState>,
    terminated: bool,
}

struct JobState {
    action: Box<FnMut() + Send + 'static>,
    last_execution_timestamp: DateTime<Utc>,
    trigger_duration: Option<Duration>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            state: Arc::new(Mutex::new(State {
                jobs: HashMap::new(),
                terminated: false,
            }))
        }
    }

    pub fn prepare<F>(&self, job_name: &str, action: F) where F: FnMut() + Send + 'static {
        let mut g = self.state.lock().unwrap();
        debug!("Preparing job '{}' for scheduling", job_name);
        g.jobs.insert(job_name.into(), JobState {
            action: Box::new(action),
            last_execution_timestamp: util::past_timestamp(),
            trigger_duration: None,
        });
    }

    pub fn schedule(&self, job_name: &str, duration: Duration) {
        let mut g = self.state.lock().unwrap();
        if let Some(ref mut js) = g.jobs.get_mut(job_name) {
            debug!("Scheduling job '{}' to be performed every {}", job_name, duration);
            js.trigger_duration = Some(duration);
        } else {
            warn!("Job '{}' does not exist, cannot schedule it", job_name);
        }
    }

    pub fn simulate(&self, job_name: &str) {
        trace!("Resetting last execution time of scheduled job '{}' to now", job_name);
        let mut g = self.state.lock().unwrap();
        if let Some(ref mut js) = g.jobs.get_mut(job_name) {
            js.last_execution_timestamp = Utc::now();
        } else {
            warn!("Job '{}' does not exist, cannot simulate it", job_name);
        }
    }

    pub fn get_last_execution_timestamp(&self, job_name: &str) -> Option<DateTime<Utc>> {
        let g = self.state.lock().unwrap();
        if let Some(ref js) = g.jobs.get(job_name) {
            trace!("Retrieving the last execution time of job '{}'", job_name);
            Some(js.last_execution_timestamp)
        } else {
            warn!("Job '{}' does not exist, cannot get its last execution time", job_name);
            None
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        info!("Starting the scheduler thread");
        let state = self.state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(StdDuration::from_secs(1));
                let mut g = state.lock().unwrap();

                if g.terminated { break; }

                let current_timestamp = Utc::now();
                for (job_name, job_state) in &mut g.jobs {
                    if let Some(trigger_duration) = job_state.trigger_duration {
                        let diff = current_timestamp.signed_duration_since(job_state.last_execution_timestamp);
                        if diff >= trigger_duration {
                            debug!("Executing scheduled job '{}'", job_name);
                            job_state.last_execution_timestamp = current_timestamp;
                            (job_state.action)();
                        }
                    }
                }
            }
        })
    }

    pub fn stop(&self) {
        info!("Stopping the scheduler thread");
        let mut g = self.state.lock().unwrap();
        g.terminated = true;
    }
}
