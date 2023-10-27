multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AdminModule {
    #[only_owner]
    #[endpoint(addUserToAdminList)]
    fn add_to_admin_list(&self, address: ManagedAddress) {
        self.admin_list().add(&address);
    }

    #[only_owner]
    #[endpoint(removeFromAdminList)]
    fn remove_from_admin_list(&self, address: ManagedAddress) {
        self.admin_list().remove(&address);
    }

    fn require_caller_is_admin(&self) {
        let caller = self.blockchain().get_caller();
        let sc_owner = self.blockchain().get_owner_address();
        if caller == sc_owner {
            return;
        }

        self.admin_list().require_whitelisted(&caller);
    }

    #[storage_mapper("adminList")]
    fn admin_list(&self) -> WhitelistMapper<ManagedAddress>;
}