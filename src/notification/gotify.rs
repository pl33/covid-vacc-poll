/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use reqwest;
use std::{error::Error};
use crate::notification::Notificator;
use async_std::task;
use crate::config::GotifySettings;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Gotify {
    url: String,
    application_token: String,
    client: reqwest::Client
}

impl Gotify {
    pub fn new(url: &String, application_token: &String) -> Gotify {
        Gotify{
            url: url.clone(),
            application_token: application_token.clone(),
            client: reqwest::Client::new()
        }
    }

    pub fn from(settings: &GotifySettings) -> Gotify {
        Gotify::new(&settings.url, &settings.application_token)
    }

    pub async fn send_message(&self, title: &str, message: &str, priority: u16) -> Result<(), Box<dyn Error>> {
        let uri = format!("{}/message?token={}", self.url, self.application_token);
        let priority = priority.to_string();
        let mut params = HashMap::new();
        params.insert("title", title);
        params.insert("message", message);
        params.insert("priority", priority.as_str());
        self.client.post(&uri).form(&params).send().await?;
        Ok(())
    }

    pub fn send_message_blocking(&self, title: &str, message: &str, priority: u16) -> Result<(), Box<dyn Error>> {
        task::block_on(self.send_message(title, message, priority))
    }
}

impl Notificator for Gotify {
    fn send_normal(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
        self.send_message_blocking(title, message, 1)
    }

    fn send_urgent(&self, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
        self.send_message_blocking(title, message, 9)
    }
}
