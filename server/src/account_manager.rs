// i think i will store accounts in a ron file for now, idc abt security for now, it's just for learning
// ig i need a fix file structure if i want to have mutiple accounts and each accound has a folder when i write their files in ?

const ACCOUNT_FILE_PATH: crate::file::ConsPath =
    crate::file::ConsPath::new(crate::file::FileSystem::External, "accounts.ron");

#[derive(Debug)]
pub struct AccountManager {
    accounts: Vec<Account>,
    pub connected_accounts: Vec<uuid::Uuid>,
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
    Connection(std::net::SocketAddr),
    Disconnection,
}

impl AccountManager {
    pub fn new_empty() -> Self {
        Self {
            accounts: Vec::new(),
            connected_accounts: Vec::new(),
        }
    }

    /// Load the account list from filesystem and returns it, returning an empty AccountManager on error
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
    /// saves the current account list to filesystem
    pub fn save(&self) {
        let pretty = ron::ser::PrettyConfig::new()
            // .depth_limit(2)
            .struct_names(false)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = ron::ser::to_string_pretty(&self.accounts, pretty).expect("Serialization failed");

        crate::file::write_bytes(ACCOUNT_FILE_PATH.into(), s).unwrap();
    }

    /// registers a new account to account list
    pub fn register(&mut self, new_account: Account) -> Result<(), ()> {
        self.accounts.push(new_account);

        Ok(())
    }

    /// Check if the username and password are corresponding to an account, logging it as connection and returning its id if success, retuning an error if fails
    pub fn login(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
        ip: std::net::SocketAddr,
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

            debug!("Loging in ({}),", username);

            return Ok(account.id);
        }

        Err(crate::error::AccountLoginError::UnknownUsername)
    }
    /// logs the id out of its account, retuning an error if it fails
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

            debug!("Loging out ({})", account.username);

            self.connected_accounts.remove(index);

            Ok(())
        } else {
            Err(crate::error::AccountLogoutError::NotLoggedIn)
        }
    }

    pub fn exit_cleanup(&mut self) {
        while !self.connected_accounts.is_empty() {
            let id = *self.connected_accounts.get(0).unwrap();
            if let Err(e) = self.logout(id) {
                error!("Tried to disconnect {id} but ecnoutered error: {e}")
            }
        }

        self.save()
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
