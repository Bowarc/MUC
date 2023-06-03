// i think i will store accounts in a ron file for now, idc abt security for now, it's just for learning
// ig i need a fix file structure if i want to have mutiple accounts and each accound has a folder when i write their files in ?

const ACCOUNT_FILE_PATH: crate::file::ConsPath =
    crate::file::ConsPath::new(crate::file::FileSystem::External, "accounts.ron");

pub struct AccountManager {
    accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    id: uuid::Uuid,
    username: String,
    password: String,
}

impl AccountManager {
    pub fn load() -> Self {
        let account_list =
            ron::de::from_bytes::<Vec<Account>>(&crate::file::load_bytes(ACCOUNT_FILE_PATH.into()))
                .unwrap();

        Self {
            accounts: account_list,
        }
    }

    pub fn save(&self) {}
}

impl Account {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            username: username.into(),
            password: password.into(),
        }
    }
}
