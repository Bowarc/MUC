// i think i will store accounts in a ron file for now, idc abt security for now, it's just for learning
// ig i need a fix file structure if i want to have mutiple accounts and each accound has a folder when i write their files in ?

const ACCOUNT_FILE_PATH: crate::file::ConsPath =
    crate::file::ConsPath::new(crate::file::FileSystem::External, "accounts.ron");

#[derive(Debug)]
pub struct AccountManager {
    accounts: Vec<Account>,
    connected_accounts: Vec<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    id: uuid::Uuid,
    username: String,
    password: String,
    logs: Vec<(chrono::DateTime<chrono::offset::Utc>, AccountLog)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountLog {
    Connection(std::net::IpAddr),
    Disconnection,
}

impl AccountManager {
    pub fn new_empty() -> Self {
        Self {
            accounts: Vec::new(),
            connected_accounts: Vec::new(),
        }
    }

    pub fn load() -> Self {
        // Could do Result<Self, Self> but meh
        if let Ok(account_list) =
            ron::de::from_bytes::<Vec<Account>>(&crate::file::load_bytes(ACCOUNT_FILE_PATH.into()))
        {
            debug!("Successfully loaded account list from {ACCOUNT_FILE_PATH:?}");

            Self {
                accounts: account_list,
                connected_accounts: Vec::new(),
            }
        } else {
            error!("Could not load the account list from {ACCOUNT_FILE_PATH:?}");
            Self::new_empty()
        }
    }

    pub fn save(&self) {
        let pretty = ron::ser::PrettyConfig::new()
            // .depth_limit(2)
            .struct_names(false)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = ron::ser::to_string_pretty(&self.accounts, pretty).expect("Serialization failed");

        crate::file::write_bytes(ACCOUNT_FILE_PATH.into(), s).unwrap();
    }

    pub fn register(&mut self, new_account: Account) -> Result<(), ()> {
        self.accounts.push(new_account);

        Ok(())
    }

    pub fn login(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
        ip: std::net::IpAddr,
    ) -> Result<uuid::Uuid, crate::error::AccountLoginError> {
        let username = username.into();
        let password = password.into();

        for account in &mut self.accounts {
            if account.username != username {
                continue;
            }

            if account.password != password {
                return Err(crate::error::AccountLoginError::InvalidPassword);
            }

            if self.connected_accounts.contains(&account.id) {
                return Err(crate::error::AccountLoginError::AlreadyLoggedIn);
            }

            account.log(AccountLog::Connection(ip));

            self.connected_accounts.push(account.id);

            return Ok(account.id);
        }

        Err(crate::error::AccountLoginError::UnknownUsername)
    }
    pub fn logout(&mut self, id: uuid::Uuid) -> Result<(), crate::error::AccountLogoutError> {
        if let Some(index) = self
            .connected_accounts
            .iter()
            .position(|connected_id| *connected_id == id)
        {
            let account_index = self
                .accounts
                .iter()
                .position(|account| account.id == id)
                .ok_or(crate::error::AccountLogoutError::UnknownAccount)?;

            let account = self
                .accounts
                .get_mut(account_index)
                .ok_or(crate::error::AccountLogoutError::UnknownAccount)?;

            account.log(AccountLog::Disconnection);

            self.connected_accounts.remove(index);

            Ok(())
        } else {
            Err(crate::error::AccountLogoutError::NotLoggedIn)
        }
    }
}

impl Account {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            username: username.into(),
            password: password.into(),
            logs: Vec::new(),
        }
    }
    pub fn log(&mut self, log: AccountLog) {
        self.logs.push((chrono::offset::Utc::now(), log));
    }
}
