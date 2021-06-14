/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::Debug;
use crate::service::{ServiceProvider, PollResult};
use crate::config::Booked4usSettings;
use reqwest;
use json;
use json::{JsonValue};
use crate::json_helper;
use std::collections::{HashSet, HashMap};
use log::{info};

#[derive(Debug)]
pub struct Booked4us {
    url: String,
    client: reqwest::Client,
    free_ids: HashSet<u32>,
    details: HashMap<u32, Detail>,
}

impl Booked4us {
    pub fn from(settings: &Booked4usSettings) -> Booked4us {
        Booked4us {
            url: settings.url.clone(),
            client: reqwest::Client::new(),
            free_ids: HashSet::new(),
            details: HashMap::new(),
        }
    }

    async fn async_poll(&mut self) -> Result<PollResult, Box<dyn Error>> {
        let details = self.get_overview().await?;
        info!("Details: {:?}", details);
        let free_slots = self.extract_free_slots(&details).await?;
        info!("Free Slots: {:?}", free_slots);
        let free_set = Self::map_to_set(&free_slots);
        let res = if self.has_changed(&free_set) {
            info!("Free Slots have changed.");
            let added = self.extract_added_slots(&free_slots);
            let removed = self.extract_removed_slots(&free_set);

            let text = format!(
                "Frei gewordene Kategorien:\n{}\nAlle freien Kategorien:\n{}\nNicht mehr frei:\n{}\nURL: {}\n",
                Self::vec_to_markdown(&added),
                Self::vec_to_markdown(&Self::map_to_vec(&free_slots)),
                Self::vec_to_markdown(&removed),
                self.url
            );
            info!("{}", text);

            self.free_ids = free_set.clone();
            self.details = details.clone();

            if added.is_empty() {
                PollResult::Normal(text)
            } else {
                PollResult::Urgent(text)
            }
        } else {
            PollResult::None
        };

        Ok(res)
    }

    async fn get_overview_json(&self) -> Result<JsonValue, Box<dyn Error>> {
        let uri = format!("{}/rest-v2/api/Calendars/WithDetails", self.url);
        let resp = self.client.get(&uri).send().await?;
        let json_str = resp.text().await?;
        let obj = json::parse(&json_str)?;
        Ok(obj)
    }

    async fn get_overview(&self) -> Result<HashMap<u32, Detail>, Box<dyn Error>> {
        let overview = self.get_overview_json().await?;
        let mut details: HashMap<u32, Detail> = HashMap::new();
        for detail_json in overview["Data"].members() {
            let detail = Detail::from_json(&detail_json)?;
            details.insert(detail.id, detail);
        }
        Ok(details)
    }

    async fn first_free_slot_json(&self, id: u32) -> Result<JsonValue, Box<dyn Error>> {
        let uri = format!("{}/rest-v2/api/Calendars/{}/FirstFreeSlot", self.url, id);
        let resp = self.client.get(&uri).send().await?;
        let json_str = resp.text().await?;
        let obj = json::parse(&json_str)?;
        Ok(obj)
    }

    async fn has_free_slots(&self, id: u32) -> Result<bool, Box<dyn Error>> {
        let first_free_slot = self.first_free_slot_json(id).await?;
        let has_free: bool = !first_free_slot["Data"].is_null();
        Ok(has_free)
    }

    async fn extract_free_slots(&self, details: &HashMap<u32, Detail>) -> Result<HashMap<u32, Detail>, Box<dyn Error>> {
        let mut free_slots: HashMap<u32, Detail> = HashMap::new();
        for (id, detail) in details {
            if self.has_free_slots(*id).await? {
                free_slots.insert(*id, detail.clone());
            }
        }
        Ok(free_slots)
    }

    fn extract_added_slots(&self, free_slots: &HashMap<u32, Detail>) -> Vec<Detail> {
        let mut added: Vec<Detail> = Vec::new();
        for (id, detail) in free_slots {
            if !self.free_ids.contains(id) {
                added.push(detail.clone());
            }
        }
        added
    }

    fn map_to_set(slots: &HashMap<u32, Detail>) -> HashSet<u32> {
        let mut set: HashSet<u32> = HashSet::new();
        for (id, _) in slots {
            set.insert(*id);
        }
        set
    }

    fn map_to_vec(slots: &HashMap<u32, Detail>) -> Vec<Detail> {
        let mut vec: Vec<Detail> = Vec::new();
        for (_, detail) in slots {
            vec.push(detail.clone());
        }
        vec
    }

    fn extract_removed_slots(&self, free_set: &HashSet<u32>) -> Vec<Detail> {
        let mut removed: Vec<Detail> = Vec::new();
        let diff: HashSet<_> = self.free_ids.difference(free_set).collect();
        for (id, detail) in &self.details {
            if diff.contains(id) {
                removed.push(detail.clone());
            }
        }
        removed
    }

    fn has_changed(&self, free_set: &HashSet<u32>) -> bool {
        let diff: HashSet<_> = self.free_ids.symmetric_difference(free_set).collect();
        !diff.is_empty()
    }

    fn vec_to_markdown(slots: &Vec<Detail>) -> String {
        let mut text = String::new();
        for slot in slots {
            text = format!("{} * {} -- ID: {}\n", text, slot.name, slot.id);
        }
        text
    }
}

impl ServiceProvider for Booked4us {
    fn poll_once(&mut self) -> Result<PollResult, Box<dyn Error>> {
        async_std::task::block_on(self.async_poll())
    }
}

#[derive(Debug)]
struct Detail {
    id: u32,
    name: String,
}

impl Detail {
    fn from_json(json: &JsonValue) -> Result<Self, Box<dyn Error>> {
        let detail = Detail {
            id: json_helper::obj_to_u32(&json["Id"])?,
            name: json_helper::obj_to_str(&json["Name"])?,
        };
        Ok(detail)
    }
}

impl Clone for Detail {
    fn clone(&self) -> Self {
        Detail {
            id: self.id,
            name: self.name.clone(),
        }
    }
}
