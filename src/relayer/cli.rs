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

use clap::{App, Arg, SubCommand};

use super::commands::Command;

pub struct Cli(App<'static, 'static>);

impl Cli {
    pub fn build() -> Cli {
        Cli(App::new(crate_name!())
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
                SubCommand::with_name("generate-config")
                    .about("Generate example configuration file"),
            )
            .subcommand(
                SubCommand::with_name("completions")
                    .about("Generate shell completions")
                    .subcommand(SubCommand::with_name("bash").about("Generate Bash completions"))
                    .subcommand(
                        SubCommand::with_name("elvish").about("Generate Elvish completions"),
                    )
                    .subcommand(SubCommand::with_name("fish").about("Generate Fish completions"))
                    .subcommand(
                        SubCommand::with_name("powershell")
                            .about("Generate PowerShell completions"),
                    )
                    .subcommand(SubCommand::with_name("zsh").about("Generate Zsh completions")),
            ))
    }

    pub fn command(self) -> Command {
        let cli = Self::build();
        let mut app = cli.0;
        let matches = app.clone().get_matches();

        match matches.subcommand() {
            ("generate-config", Some(_)) => Command::GenerateConfiguration,
            ("daemon", Some(cmd)) => {
                use std::path::PathBuf;
                let config_file_path =
                    cmd.value_of("config")
                        .map(PathBuf::from)
                        .unwrap_or(match dirs::config_dir() {
                            Some(data_dir) => data_dir.join(crate_name!()).join("config.toml"),
                            None => PathBuf::from("config.toml"),
                        });

                fdlimit::raise_fd_limit();
                Command::Service {
                    config_file_path: config_file_path.to_owned(),
                    daemonize: false,
                }
            }
            ("completions", Some(cmd)) => {
                let shell = match cmd.subcommand() {
                    ("bash", _) => clap::Shell::Bash,
                    ("fish", _) => clap::Shell::Fish,
                    ("zsh", _) => clap::Shell::Zsh,
                    ("elvish", _) => clap::Shell::Elvish,
                    ("powershell", _) => clap::Shell::PowerShell,
                    _ => {
                        app.print_help().unwrap();
                        return Command::ExitFailure;
                    }
                };
                app.gen_completions_to(crate_name!(), shell, &mut std::io::stdout());
                Command::ExitSuccess
            }
            ("help", Some(_)) => {
                app.print_help().unwrap();
                Command::ExitSuccess
            }
            ("version", Some(_)) => {
                println!("{} {}", crate_name!(), crate_version!());
                Command::ExitSuccess
            }
            (_, _) => {
                app.print_help().unwrap();
                Command::ExitFailure
            }
        }
    }
}
