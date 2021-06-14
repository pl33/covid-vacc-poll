/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::notification::AdminNotifications;

mod config;
mod notification;
mod service;
mod error;
mod json_helper;

use ctrlc;
use simple_logger::SimpleLogger;
use log::{LevelFilter};
use clap;

fn main() {
    let args = clap::App::new("COVID Vaccination Poll App")
        .version("1.0.0")
        .author("Philipp Le")
        .about("Polls the available appointments for COVID vaccination")
        .arg(clap::Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Configuration JSON file"))
        .arg(clap::Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .help("Enable verbose output"))
        .get_matches();

    SimpleLogger::new().with_level(if args.is_present("verbose") {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    }).init().unwrap();

    let filename = args.value_of("config").unwrap();
    let cfg = config::Config::read_from_file(filename).unwrap();

    let notifs = notification::NotificatorCollection::from(&cfg);
    let admin_notifs = AdminNotifications::new(notifs.subcollection(&cfg.admin_notifications));
    let services = service::ServiceCollection::from(&cfg, &notifs, &admin_notifs);

    admin_notifs.get_tx().send("App", "COVID Vaccination Poll App Started");

    let service_killer = services.get_killers();
    ctrlc::set_handler(move || {
        service_killer.kill_all();
    }).unwrap();
    services.join_all();
    admin_notifs.get_tx().send("App", "COVID Vaccination Poll App Terminated");

    admin_notifs.get_killer().kill();
    admin_notifs.join().unwrap();
}
