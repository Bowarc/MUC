#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    Channel(#[from] shared::error::ChannelError<T>),
    Account(#[from] AccountError),
}

#[derive(thiserror::Error, Debug)]
pub enum AccountError {
    #[error(transparent)]
    Login(#[from] AccountLoginError),
    #[error(transparent)]
    Logout(#[from] AccountLogoutError),
    #[error(transparent)]
    Registration(#[from] AccountRegistrationError),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccountLoginError {
    #[error("Invalid password")]
    InvalidPassword,
    #[error("This username doens't exist")]
    UnknownUsername,

    #[error("This account is already being used by another machine")]
    AlreadyLoggedIn,
}
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccountLogoutError {
    #[error("The user is not logged in")]
    NotLoggedIn,
    #[error("The given id does not correspond to any account")]
    UnknownAccount,
}

#[derive(thiserror::Error, Debug)]
pub enum AccountRegistrationError {
    #[error("The chosen name is already used by another user")]
    NameAlreadyInUse(&'static str),
}
