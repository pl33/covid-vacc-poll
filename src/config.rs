/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{error::Error, fs};
use std::collections::HashMap;

use json;
use json::JsonValue;

use crate::json_helper::*;

#[derive(Debug)]
pub struct Config {
    pub admin_notifications: Vec<String>,
    pub services: Vec<ServiceSettings>,
    pub notifications: HashMap<String, NotificationSettings>
}

impl Config {
    pub fn read_from_file(filename: &str) -> Result<Config, Box<dyn Error>> {
        let json_str = fs::read_to_string(filename)?;
        let config = Config::read_from_json_str(&json_str)?;
        Ok(config)
    }

    fn read_from_json_str(str: &String) -> Result<Config, Box<dyn Error>> {
        let json_obj = json::parse(str)?;
        let config = Config::load_from_json_object(&json_obj)?;
        Ok(config)
    }

    fn load_from_json_object(obj: &JsonValue) -> Result<Config, Box<dyn Error>> {
        let config = Config{
            admin_notifications: to_str_array(&obj["admin_notifications"])?,
            services: {
                let mut srv: Vec<ServiceSettings> = Vec::new();
                for content in obj["services"].members() {
                    let settings = ServiceSettings::load_from_json_object(&content)?;
                    srv.push(settings);
                }
                srv
            },
            notifications: {
                let mut notifs: HashMap<String, NotificationSettings> = HashMap::new();
                for (key, content) in obj["notifications"].entries() {
                    let settings = NotificationSettings::load_from_json_object(&content)?;
                    notifs.insert(String::from(key), settings);
                }
                notifs
            }
        };
        Ok(config)
    }
}

#[derive(Debug)]
pub enum ServiceProviderSettings {
    Booked4us(Booked4usSettings)
}

#[derive(Debug)]
pub struct ServiceSettings {
    pub provider: ServiceProviderSettings,
    pub notifications: Vec<String>,
    pub sleep: u32,
    pub title: String
}

impl ServiceSettings {
    fn load_from_json_object(obj: &JsonValue) -> Result<ServiceSettings, Box<dyn Error>> {
        let provider = obj_to_str(&obj["provider"])?;
        let srv: ServiceProviderSettings = match provider.as_str() {
            "booked4us" => ServiceProviderSettings::Booked4us(Booked4usSettings::load_from_json_object(&obj["settings"])?),
            _ => return Err(ParseError::new("services[].provider is invalid"))
        };
        let notifications = to_str_array(&obj["notifications"])?;
        Ok(ServiceSettings{
            provider: srv,
            notifications,
            sleep: obj_to_u32(&obj["sleep"])?,
            title: obj_to_str(&obj["title"])?
        })
    }
}

#[derive(Debug)]
pub struct Booked4usSettings {
    pub url: String
}

impl Booked4usSettings {
    fn load_from_json_object(obj: &JsonValue) -> Result<Booked4usSettings, Box<dyn Error>> {
        let settings = Booked4usSettings{
            url: obj_to_str(&obj["url"])?
        };
        Ok(settings)
    }
}

#[derive(Debug)]
pub enum NotificationSettings {
    Email(EmailSettings),
    Gotify(GotifySettings)
}

impl NotificationSettings {
    fn load_from_json_object(obj: &JsonValue) -> Result<NotificationSettings, Box<dyn Error>> {
        let provider = obj_to_str(&obj["provider"])?;
        let notif: NotificationSettings = match provider.as_str() {
            "email" => NotificationSettings::Email(EmailSettings::load_from_json_object(&obj["settings"])?),
            "gotify" => NotificationSettings::Gotify(GotifySettings::load_from_json_object(&obj["settings"])?),
            _ => return Err(ParseError::new("notifications[].provider is invalid"))
        };
        Ok(notif)
    }
}

#[derive(Debug)]
pub struct EmailSettings {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_starttls: bool
}

impl EmailSettings {
    fn load_from_json_object(obj: &JsonValue) -> Result<EmailSettings, Box<dyn Error>> {
        let settings = EmailSettings{
            from: obj_to_str(&obj["from"])?,
            subject: obj_to_str(&obj["subject"])?,
            smtp_host: obj_to_str(&obj["smtp"]["host"])?,
            smtp_port: obj_to_u16(&obj["smtp"]["port"])?,
            smtp_user: obj_to_str(&obj["smtp"]["user"])?,
            smtp_password: obj_to_str(&obj["smtp"]["password"])?,
            smtp_starttls: obj_to_bool(&obj["smtp"]["starttls"])?,
            to: to_str_array(&obj["to"])?
        };
        Ok(settings)
    }
}

#[derive(Debug)]
pub struct GotifySettings {
    pub url: String,
    pub application_token: String
}

impl GotifySettings {
    fn load_from_json_object(obj: &JsonValue) -> Result<GotifySettings, Box<dyn Error>> {
        let settings = GotifySettings{
            url: obj_to_str(&obj["url"])?,
            application_token: obj_to_str(&obj["application_token"])?
        };
        Ok(settings)
    }
}
