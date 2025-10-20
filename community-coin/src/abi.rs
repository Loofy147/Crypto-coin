//! The Application Binary Interface (ABI) for Community Coin smart contracts.

/// The functions that a smart contract can import from the blockchain environment.
#[derive(Debug, Clone, Copy)]
pub enum Abi {
    /// Get the balance of an address.
    ///
    /// # Arguments
    ///
    /// * `address_ptr` - A pointer to the address in the contract's memory.
    /// * `address_len` - The length of the address.
    ///
    /// # Returns
    ///
    /// The balance of the address.
    GetBalance,
    /// Transfer coins to an address.
    ///
    /// # Arguments
    ///
    /// * `to_ptr` - A pointer to the recipient's address in the contract's memory.
    /// * `to_len` - The length of the recipient's address.
    /// * `amount` - The amount of coins to transfer.
    Transfer,
    /// Get a value from the contract's storage.
    ///
    /// # Arguments
    ///
    /// * `key_ptr` - A pointer to the key in the contract's memory.
    /// * `key_len` - The length of the key.
    /// * `value_ptr` - A pointer to a buffer in the contract's memory to write the value to.
    /// * `value_len` - The length of the value buffer.
    ///
    /// # Returns
    ///
    /// The number of bytes written to the value buffer.
    GetStorage,
    /// Set a value in the contract's storage.
    ///
    /// # Arguments
    ///
    /// * `key_ptr` - A pointer to the key in the contract's memory.
    /// * `key_len` - The length of the key.
    /// * `value_ptr` - A pointer to the value in the contract's memory.
    /// * `value_len` - The length of the value.
    SetStorage,
}
