use std::time::{Duration, Instant};

use anyhow::anyhow;
use beas_bsl::{Client, ClientConfig};
use chrono::{Datelike, Local, Timelike};

mod types;
pub use types::{State, StateOneData, StateTwoData};

#[derive(Debug)]
pub struct StageOneService {
    // config
    pub enabled: bool,
    pub request_timeout: Duration,

    // state
    pub state: State,
    pub client: Option<Client>,
    pub last_request_ts: Instant,
}

// public interface
impl StageOneService {
    pub fn new(request_timeout: Duration) -> Self {
        Self { 
            enabled: false, 
            client: None, 
            state: State::Zero,
            request_timeout, 
            last_request_ts: Instant::now(), 
        }
    }

    pub fn set_enabled(&mut self, value: bool) {
        if self.enabled == value { return; }
        self.enabled = value;
    }

    pub fn connect(&mut self, config_path: &str) -> anyhow::Result<()> {
        if self.client.is_some() {
            return Ok(());
        }

        self.state = State::Zero;

        let config = ClientConfig::from_file(config_path)
            .map_err(|e| anyhow!("[ServiceS1] Failed to read Config: {:?}", e))?;

        let client = Client::new(config)
            .map_err(|e| anyhow!("[ServiceS1] Failed to create Client: {:?}", e))?;

        self.client = Some(client);
        return Ok(())
    }

    pub fn disconnect(&mut self) {
        self.state = State::Zero;
        self.client = None;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn update(&mut self, now: Instant, plate_count: u32) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let Some(client) = &mut self.client else {
            return Ok(());
        };

        if client.has_pending_request() {
            return Ok(());
        } 

        return Ok(());
    }
}