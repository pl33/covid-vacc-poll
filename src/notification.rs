/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{error::Error, thread};
use std::collections::HashMap;
use std::fmt::Debug;
use log::error;

use gotify::Gotify;

use crate::config::{Config, NotificationSettings};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use crate::error::GenericError;

mod gotify;

pub trait Notificator: Debug + Send + Sync {
    fn send_normal(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>>;
    fn send_urgent(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct NotificatorCollection {
    notificators: HashMap<String, Arc<Mutex<dyn Notificator>>>
}

impl NotificatorCollection {
    fn new() -> NotificatorCollection {
        NotificatorCollection{
            notificators: HashMap::new()
        }
    }

    fn add(&mut self, name: &String, provider: Arc<Mutex<dyn Notificator>>) {
        self.notificators.insert(name.clone(), provider);
    }

    pub fn from(config: &Config) -> NotificatorCollection {
        let mut coll = NotificatorCollection::new();
        for (name, settings) in config.notifications.iter() {
            let notif = match settings {
                NotificationSettings::Gotify(s) => Arc::new(Mutex::new(Gotify::from(s))),
                NotificationSettings::Email(_) => Arc::new(Mutex::new(Gotify::new(&String::from(""), &String::from(""))))
            };
            coll.add(name, notif);
        }
        coll
    }

    // pub fn get(&self, name: &String) -> Arc<Mutex<dyn Notificator>> {
    //     self.notificators[name].clone()
    // }

    pub fn subcollection(&self, names: &Vec<String>) -> NotificatorSubCollection {
        let mut arr: Vec<Arc<Mutex<dyn Notificator>>> = Vec::new();
        for name in names {
            arr.push(self.notificators[name].clone());
        }
        NotificatorSubCollection{
            notificators: arr
        }
    }
}

#[derive(Debug)]
pub struct NotificatorSubCollection {
    notificators: Vec<Arc<Mutex<dyn Notificator>>>
}

impl Notificator for NotificatorSubCollection {
    fn send_normal(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
        for notif in self.notificators.iter() {
            match notif.lock() {
                Ok(l) => l,
                Err(err) => return Err(Box::new(GenericError::new(err.to_string().as_str())))
            }.send_normal(title, message)?;
        }
        Ok(())
    }

    fn send_urgent(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
        for notif in self.notificators.iter() {
            match notif.lock() {
                Ok(l) => l,
                Err(err) => return Err(Box::new(GenericError::new(err.to_string().as_str())))
            }.send_urgent(title, message)?;
        }
        Ok(())
    }
}

pub struct AdminNotifications {
    thrd: thread::JoinHandle<()>,
    kill_tx: mpsc::Sender<bool>,
    msg_tx: mpsc::Sender<String>
}

impl AdminNotifications {
    pub fn new(notificators: NotificatorSubCollection) -> AdminNotifications {
        let (msg_tx, msg_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        let (kill_tx, kill_rx) = mpsc::channel();
        let thrd = thread::spawn(move || {
            let mut running = true;
            while running {
                thread::sleep(Duration::from_secs(1));
                match msg_rx.try_recv() {
                    Ok(msg) => match notificators.send_normal("COVID Vaccination Poll - Admin", msg.as_str()) {
                        Ok(_) => (),
                        Err(error) => error!("{}", error.to_string().as_str())
                    },
                    Err(_) => ()
                }
                match kill_rx.try_recv() {
                    Ok(_) => { running = false; },
                    Err(_) => ()
                }
            }
        });
        AdminNotifications{
            thrd,
            kill_tx,
            msg_tx
        }
    }

    pub fn get_killer(&self) -> AdminNotificationsKiller {
        AdminNotificationsKiller{
            kill_tx: self.kill_tx.clone()
        }
    }

    pub fn join(self) -> thread::Result<()> {
        self.thrd.join()
    }

    pub fn get_tx(&self) -> AdminNotificationsSender {
        AdminNotificationsSender {
            msg_tx: self.msg_tx.clone()
        }
    }
}

pub struct AdminNotificationsKiller {
    kill_tx: mpsc::Sender<bool>
}

impl AdminNotificationsKiller {
    pub fn kill(&self) {
        self.kill_tx.send(true).unwrap();
    }
}

pub struct AdminNotificationsSender {
    msg_tx: mpsc::Sender<String>
}

impl AdminNotificationsSender {
    pub fn send(&self, title: &str, message: &str) {
        let msg = format!("{}: {}", title, message);
        self.msg_tx.send(msg).unwrap();
    }
}

impl Clone for AdminNotificationsSender {
    fn clone(&self) -> Self {
        AdminNotificationsSender {
            msg_tx: self.msg_tx.clone()
        }
    }
}
