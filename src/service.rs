/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod booked4us;

use std::error::Error;
use std::fmt::Debug;
// use std::fmt::Display;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use crate::config::{Config, ServiceProviderSettings};
use booked4us::Booked4us;
use crate::notification::{NotificatorSubCollection, NotificatorCollection, Notificator, AdminNotificationsSender, AdminNotifications};
use std::time::Duration;
use log::{info, error};

pub enum PollResult {
    None,
    Normal(String),
    Urgent(String)
}

pub trait ServiceProvider: Debug + Send + Sync {
    fn poll_once(&mut self) -> Result<PollResult, Box<dyn Error>>;
}

#[derive(Debug)]
pub struct Service {
    thrd: thread::JoinHandle<()>,
    kill_tx: mpsc::Sender<bool>
}

impl Service {
    pub fn new(title: String, provider: Arc<Mutex<dyn ServiceProvider>>, notifications: NotificatorSubCollection, sleep: u32, admin_notif: AdminNotificationsSender) -> Service {
        let (kill_tx, kill_rx) = mpsc::channel();
        let thrd = thread::spawn(move || {
            let mut running = true;
            while running {
                let mut locked_provider = provider.lock().unwrap();

                info!("Polling {}", title);
                match locked_provider.poll_once() {
                    Ok(result) => match result {
                        PollResult::Urgent(msg) => match notifications.send_urgent(title.as_str(), msg.as_str()) {
                            Ok(_) => (),
                            Err(error) => {
                                error!("{}: {}", title.as_str(), error.to_string().as_str());
                                admin_notif.send(title.as_str(), error.to_string().as_str())
                            }
                        },
                        PollResult::Normal(msg) => match notifications.send_normal(title.as_str(), msg.as_str()) {
                            Ok(_) => (),
                            Err(error) => {
                                error!("{}: {}", title.as_str(), error.to_string().as_str());
                                admin_notif.send(title.as_str(), error.to_string().as_str())
                            }
                        },
                        PollResult::None => ()
                    },
                    Err(error) => {
                        error!("{}: {}", title.as_str(), error.to_string().as_str());
                        admin_notif.send(title.as_str(), error.to_string().as_str())
                    }
                }

                info!("Sleeping. Next poll of {} in {} s.", title, sleep);
                'sleep: for _index in 0..sleep {
                    thread::sleep(Duration::from_secs(1));
                    match kill_rx.try_recv() {
                        Ok(_) => {
                            running = false;
                            break 'sleep;
                        },
                        Err(_) => ()
                    }
                }
            }
        });
        Service{
            thrd,
            kill_tx
        }
    }

    pub fn get_killer(&self) -> mpsc::Sender<bool> {
        self.kill_tx.clone()
    }

    pub fn join(self) -> thread::Result<()> {
        self.thrd.join()
    }
}

#[derive(Debug)]
pub struct ServiceCollection {
    services: Vec<Service>
}

impl ServiceCollection {
    fn new() -> Self {
        ServiceCollection{
            services: Vec::new()
        }
    }

    fn add(&mut self, service: Service) {
        self.services.push(service)
    }

    pub fn from(config: &Config, notificators: &NotificatorCollection, admin_notif: &AdminNotifications) -> Self {
        let mut coll = ServiceCollection::new();
        for settings in config.services.iter() {
            let provider = Arc::new(
                Mutex::new(match &settings.provider {
                    ServiceProviderSettings::Booked4us(s) => Booked4us::from(s)
                })
            );
            let notifications = notificators.subcollection(&settings.notifications);
            coll.add(Service::new(settings.title.clone(), provider, notifications, settings.sleep, admin_notif.get_tx()));
        }
        coll
    }

    pub fn get_killers(&self) -> ServiceKillers {
        ServiceKillers{
            kill_tx: {
                let mut v: Vec<mpsc::Sender<bool>> = Vec::new();
                for srv in &self.services {
                    v.push(srv.get_killer());
                }
                v
            }
        }
    }

    pub fn join_all(mut self) {
        while !self.services.is_empty() {
            match self.services.pop() {
                Some(srv) => srv.join().unwrap(),
                None => ()
            }
        }
    }
}

pub struct ServiceKillers {
    kill_tx: Vec<mpsc::Sender<bool>>
}

impl ServiceKillers {
    pub fn kill_all(&self) {
        for tx in &self.kill_tx {
            tx.send(true).unwrap();
        }
    }
}

// #[derive(Debug)]
// pub struct PollError {
//     msg: String
// }
//
// impl Error for PollError {}
//
// impl Display for PollError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Config parsing error: {}", self.msg)
//     }
// }
//
// impl PollError {
//     fn new(s: &str) -> Box<Self> {
//         Box::new(Self{msg: String::from(s)})
//     }
// }
