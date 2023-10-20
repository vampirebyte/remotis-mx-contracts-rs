#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc_modules::{default_issue_callbacks, subscription};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct FixedAttributes {
    pub boosts: u64,
    pub huds: u64,
    pub sounds: u64,
    pub max_rpm: u64,
}

#[derive(TypeAbi, NestedEncode, NestedDecode)]
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
pub trait MxContractsRs: default_issue_callbacks::DefaultIssueCallbacksModule + subscription::SubscriptionModule {
    
    #[init]
    fn init(&self) {}
    
    #[only_owner]
    #[endpoint(setFixedAttributes)]
    fn set_fixed_attributes(&self, license_type: u8, boosts: u64, huds: u64, sounds: u64, max_rpm: u64) {
        self.fixed_attributes(license_type).set(FixedAttributes {
            boosts,
            huds,
            sounds,
            max_rpm,
        })
    }
        
    #[only_owner]
    #[payable("*")]
    #[endpoint(issueCollection)]
    fn issue_collection(&self) {
        self.token_id().issue_and_set_all_roles(
            EsdtTokenType::NonFungible,
            self.call_value().egld_value().clone_value(),
            ManagedBuffer::from(b"Subscription"),
            ManagedBuffer::from(b"SUB"),
            0,
            None,
        )
    }

    #[only_owner]
    #[endpoint(mintLicense)]
    fn mint_license(&self, recipient: ManagedAddress, license: u8, duration: u64, battery: u64, renewable: bool, rechargeable: bool) {
        require!(!self.fixed_attributes(license).is_empty(), "Set fixed attributes");
        let stored = self.fixed_attributes(license).get();

        let license_type = match license {
            1 => ManagedBuffer::from(b"Amateur"),
            2 => ManagedBuffer::from(b"Specialist"),
            3 => ManagedBuffer::from(b"Expert"),
            4 => ManagedBuffer::from(b"Legend"),
            _ => sc_panic!("Wrong License type"),
        };

        let mut uris = ManagedVec::new();
        let uri = match license {
            1 => ManagedBuffer::from(b"https://ipfs.io/ipfs/QmaSBad87GFUaLXi1FjgkiqRyQ3Fbc5GENNtzEdKYro3y4"),
            2 => ManagedBuffer::from(b"https://ipfs.io/ipfs/QmSRkhriLVWBLATWC946JXYdy2zX1jSSNAce4zRanTBdqY"),
            3 => ManagedBuffer::from(b"https://ipfs.io/ipfs/QmbAEp9VFGV82UiL1LMrki46fZ5nD7AEpr7L5NzYNz1nEw"),
            4 => ManagedBuffer::from(b"https://ipfs.io/ipfs/QmZR4sKhySN2nYftJu7hCCJ9UteXgNDuZhTDyXEfaZFimY"),
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
            self.token_id().get_token_id_ref(),
            &BigUint::from(1u8),
            &ManagedBuffer::from(b"Remotis Racing License"),
            &BigUint::from(10u8),
            &ManagedBuffer::new(),
            duration,
            attributes,
            &uris,
        );
        self.send().direct_esdt(
            &recipient,
            self.token_id().get_token_id_ref(),
            nonce,
            &BigUint::from(1u8),
        );
    }

    #[only_owner]
    #[endpoint(freezeLicense)]
    fn freeze_license(&self, address: ManagedAddress) {
        unimplemented!()
    }

    #[only_owner]
    #[endpoint(unfreezeLicense)]
    fn unfreeze_license(&self, address: ManagedAddress) {
        unimplemented!()
    }

    #[only_owner]
    #[endpoint(wipeLicense)]
    fn wipe_license(&self, address: ManagedAddress) {
        unimplemented!()
    }

    #[only_owner]
    #[endpoint(setRoleToContract)]
    fn set_role_to_contract(&self, address: ManagedAddress) {
        unimplemented!()
    }

    #[view(getTokenId)]
    #[storage_mapper("tokenId")]
    fn token_id(&self) -> NonFungibleTokenMapper;

    #[view(getFixedAttributes)]
    #[storage_mapper("fixedAttributes")]
    fn fixed_attributes(&self, license_type: u8) -> SingleValueMapper<FixedAttributes>;
}
