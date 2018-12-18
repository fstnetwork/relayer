// Copyright 2017-2018 FST Network Pte. Ltd.
// This file is part of FST Relayer.

// FST Relayer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// FST Relayer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with FST Relayer. If not, see <http://www.gnu.org/licenses/>.
mod error;

use clap::{App, Arg, SubCommand};

pub fn build() -> App<'static, 'static> {
    App::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("help").about("Show usage of FST Relayer"))
        .subcommand(SubCommand::with_name("version").about("Show version of FST Relayer"))
        .subcommand(
            SubCommand::with_name("service")
                .about("Run FST Relayer Service")
                .arg(
                    Arg::with_name("config")
                        .long("config")
                        .short("c")
                        .takes_value(true)
                        .help("The configuration file used by FST Relayer"),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate-config").about("Generate example configuration file"),
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate shell completions")
                .subcommand(SubCommand::with_name("bash").about("Generate Bash completions"))
                .subcommand(SubCommand::with_name("fish").about("Generate Fish completions"))
                .subcommand(SubCommand::with_name("zsh").about("Generate Zsh completions"))
                .subcommand(
                    SubCommand::with_name("powershell").about("Generate PowerShell completions"),
                ),
        )
}
