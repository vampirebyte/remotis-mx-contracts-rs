#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod admin;

use multiversx_sc_modules::subscription::{self, SubscriptionAttributes};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct FixedAttributes {
    pub boosts: u64,
    pub huds: u64,
    pub sounds: u64,
    pub max_rpm: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Attributes<M: ManagedTypeApi> {
    pub boosts: u64,
    pub huds: u64,
    pub sounds: u64,
    pub races_won: u64,
    pub max_rpm: u64,
    pub battery: u64,
    pub license_type: ManagedBuffer<M>,
    pub renewable: bool,
    pub rechargeable: bool,
}

#[multiversx_sc::contract]
#[esdt_attribute("RRL", SubscriptionAttributes<Attributes<Self::Api>>)]
pub trait MxContractsRs: subscription::SubscriptionModule + admin::AdminModule {
    #[init]
    fn init(&self) {}

    // Set up

    #[payable("EGLD")]
    #[endpoint(issueCollection)]
    fn issue_collection(&self) {
        self.require_caller_is_admin();
        require!(self.token_id().is_empty(), "Token already issued");

        let payment_amount = self.call_value().egld_value();
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                payment_amount.clone_value(),
                &ManagedBuffer::from(b"Licenses"),
                &ManagedBuffer::from(b"RRL"),
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_transfer_create_role: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback())
            .call_and_exit()
    }

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.token_id().set(&token_id.unwrap_esdt());
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.send()
                        .direct(&caller, &returned.token_identifier, 0, &returned.amount);
                }
            }
        }
    }

    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Token not issued");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.token_id().get(),
                [EsdtLocalRole::NftCreate][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[endpoint(setFixedAttributes)]
    fn set_fixed_attributes(
        &self,
        license_type: u8,
        boosts: u64,
        huds: u64,
        sounds: u64,
        max_rpm: u64,
    ) {
        self.require_caller_is_admin();
        self.fixed_attributes(license_type).set(FixedAttributes {
            boosts,
            huds,
            sounds,
            max_rpm,
        })
    }

    // create and send license

    #[endpoint(mintLicense)]
    fn mint_license(
        &self,
        recipient: ManagedAddress,
        license: u8,
        duration: u64,
        battery: u64,
        renewable: bool,
        rechargeable: bool,
    ) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Collection not issued");
        require!(
            !self.fixed_attributes(license).is_empty(),
            "Set fixed attributes"
        );

        let stored = self.fixed_attributes(license).get();

        let license_type = match license {
            1 => ManagedBuffer::from(b"Remotis Amateur License"),
            2 => ManagedBuffer::from(b"Remotis Specialist License"),
            3 => ManagedBuffer::from(b"Remotis Expert License"),
            4 => ManagedBuffer::from(b"Remotis Legend License"),
            _ => sc_panic!("Wrong License type"),
        };

        let mut uris = ManagedVec::new();
        let uri = match license {
            1 => ManagedBuffer::from(
                b"https://ipfs.io/ipfs/QmaSBad87GFUaLXi1FjgkiqRyQ3Fbc5GENNtzEdKYro3y4",
            ),
            2 => ManagedBuffer::from(
                b"https://ipfs.io/ipfs/QmSRkhriLVWBLATWC946JXYdy2zX1jSSNAce4zRanTBdqY",
            ),
            3 => ManagedBuffer::from(
                b"https://ipfs.io/ipfs/QmbAEp9VFGV82UiL1LMrki46fZ5nD7AEpr7L5NzYNz1nEw",
            ),
            4 => ManagedBuffer::from(
                b"https://ipfs.io/ipfs/QmZR4sKhySN2nYftJu7hCCJ9UteXgNDuZhTDyXEfaZFimY",
            ),
            _ => sc_panic!("Wrong License type"),
        };
        uris.push(uri);

        let attributes = Attributes {
            boosts: stored.boosts,
            huds: stored.huds,
            sounds: stored.sounds,
            races_won: 0,
            max_rpm: stored.max_rpm,
            battery,
            license_type,
            renewable,
            rechargeable,
        };

        let nonce = self.create_subscription_nft(
            &self.token_id().get(),
            &BigUint::from(1u8),
            &ManagedBuffer::from(b"Remotis Racing License"),
            &BigUint::from(1000u32),
            &ManagedBuffer::new(),
            duration,
            attributes,
            &uris,
        );
        self.send().direct_esdt(
            &recipient,
            &self.token_id().get(),
            nonce,
            &BigUint::from(1u8),
        );
    }

    // management functions

    #[endpoint(freezeLicense)]
    fn freeze_license(&self, nonce: u64, address: &ManagedAddress) {
        self.require_caller_is_admin();
        self.send()
            .esdt_system_sc_proxy()
            .freeze_nft(&self.token_id().get(), nonce, address)
            .async_call()
            .call_and_exit();
    }

    #[endpoint(unfreezeLicense)]
    fn unfreeze_license(&self, nonce: u64, address: &ManagedAddress) {
        self.require_caller_is_admin();
        self.send()
            .esdt_system_sc_proxy()
            .unfreeze_nft(&self.token_id().get(), nonce, address)
            .async_call()
            .call_and_exit();
    }

    #[endpoint(wipeLicense)]
    fn wipe_license(&self, nonce: u64, address: &ManagedAddress) {
        self.require_caller_is_admin();
        self.send()
            .esdt_system_sc_proxy()
            .wipe_nft(&self.token_id().get(), nonce, address)
            .async_call()
            .call_and_exit();
    }

    #[endpoint(setUpdateAttributesRoleTo)]
    fn set_update_attributes_role_to(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Collection not issue");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &address,
                &self.token_id().get(),
                [EsdtLocalRole::NftUpdateAttributes][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[endpoint(setMintRoleTo)]
    fn set_mint_role_to(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Collection not issue");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &address,
                &self.token_id().get(),
                [EsdtLocalRole::NftCreate][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[endpoint(setBurnRoleTo)]
    fn set_burn_role_to(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Collection not issue");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &address,
                &self.token_id().get(),
                [EsdtLocalRole::NftBurn][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[endpoint(setAddUrisRoleTo)]
    fn set_add_uris_role_to(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        require!(!self.token_id().is_empty(), "Collection not issue");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &address,
                &self.token_id().get(),
                [EsdtLocalRole::NftAddUri][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[view(getAttributes)]
    fn get_attributes(&self, nonce: u64) -> Attributes<Self::Api> {
        self.get_subscription_attributes(&self.token_id().get(), nonce)
    }

    // storage

    #[view(getTokenId)]
    #[storage_mapper("tokenId")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getFixedAttributes)]
    #[storage_mapper("fixedAttributes")]
    fn fixed_attributes(&self, license_type: u8) -> SingleValueMapper<FixedAttributes>;
}
