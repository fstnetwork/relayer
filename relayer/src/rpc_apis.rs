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
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Api {
    Admin,
    Network,
    Pool,
    Relayer,
    SystemInfo,
    Token,
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Api::*;

        match s {
            "admin" => Ok(Admin),
            "network" => Ok(Network),
            "pool" => Ok(Pool),
            "relayer" => Ok(Relayer),
            "system" => Ok(SystemInfo),
            "token" => Ok(Token),
            api => Err(format!("Unknown api: {}", api)),
        }
    }
}

impl ToString for Api {
    fn to_string(&self) -> String {
        match self {
            Api::Admin => "admin".to_owned(),
            Api::Network => "network".to_owned(),
            Api::Pool => "pool".to_owned(),
            Api::Relayer => "relayer".to_owned(),
            Api::SystemInfo => "system".to_owned(),
            Api::Token => "token".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiSet {
    All,
    List(HashSet<Api>),
}

impl ::std::fmt::Display for ApiSet {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let mut v: Vec<_> = self.apis().iter().map(Api::to_string).collect();
        v.sort_unstable();
        write!(f, "{:?}", v)
    }
}

impl ApiSet {
    pub fn new_all() -> ApiSet {
        ApiSet::All
    }

    pub fn new_list() -> ApiSet {
        ApiSet::List(HashSet::new())
    }

    pub fn with_apis(apis: HashSet<Api>) -> ApiSet {
        ApiSet::List(apis)
    }

    pub fn add(&mut self, api: Api) {
        match self {
            ApiSet::List(set) => {
                set.insert(api);
            }
            _ => {}
        }
    }

    pub fn from_strings(apis: Vec<String>) -> ApiSet {
        if apis.contains(&"all".to_owned()) {
            return ApiSet::All;
        }

        apis.iter().fold(ApiSet::new_list(), |mut api_set, api| {
            if let Ok(api) = Api::from_str(&api) {
                api_set.add(api);
            } else {
                warn!("failed to parse API {}", api);
            }
            api_set
        })
    }

    pub fn apis(&self) -> HashSet<Api> {
        match self {
            ApiSet::All => {
                let mut apis = HashSet::new();
                apis.insert(Api::Admin);
                apis.insert(Api::Network);
                apis.insert(Api::Pool);
                apis.insert(Api::Relayer);
                apis.insert(Api::SystemInfo);
                apis.insert(Api::Token);
                apis
            }
            ApiSet::List(apis) => apis.clone(),
        }
    }
}
