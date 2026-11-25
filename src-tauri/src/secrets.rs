pub(crate) trait SecretStore {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, value: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

pub(crate) struct KeyringStore;

impl KeyringStore {
    fn entry() -> Result<keyring::Entry, String> {
        keyring::Entry::new("com.luizveltroni.tradutor", "deepl-api-key")
            .map_err(|_| "Could not open the system keychain entry".into())
    }
}

impl SecretStore for KeyringStore {
    fn get(&self) -> Result<Option<String>, String> {
        match Self::entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("Could not read the DeepL key from the system keychain".into()),
        }
    }

    fn set(&self, value: &str) -> Result<(), String> {
        Self::entry()?
            .set_password(value)
            .map_err(|_| "Could not write the DeepL key to the system keychain".into())
    }

    fn delete(&self) -> Result<(), String> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Could not remove the DeepL key from the system keychain".into()),
        }
    }
}

pub(crate) fn store_verified(store: &dyn SecretStore, key: &str) -> Result<(), String> {
    store.set(key)?;
    if store.get()?.as_deref() == Some(key) {
        Ok(())
    } else {
        Err("The system keychain did not return the saved key".into())
    }
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct MemoryStore {
    pub(crate) key: std::cell::RefCell<Option<String>>,
    pub(crate) fail_set: bool,
    pub(crate) wrong_read: bool,
}

#[cfg(test)]
impl SecretStore for MemoryStore {
    fn get(&self) -> Result<Option<String>, String> {
        if self.wrong_read {
            return Ok(Some("different key".into()));
        }
        Ok(self.key.borrow().clone())
    }

    fn set(&self, value: &str) -> Result<(), String> {
        if self.fail_set {
            return Err("Unavailable".into());
        }
        self.key.replace(Some(value.into()));
        Ok(())
    }

    fn delete(&self) -> Result<(), String> {
        self.key.replace(None);
        Ok(())
    }
}
