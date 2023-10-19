// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            9
// Async Callback:                       1
// Total number of exported functions:  11

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    mx_contracts_rs
    (
        init => init
        setFixedAttributes => set_fixed_attributes
        issueCollection => issue_collection
        mintLicense => mint_license
        freezeLicense => freeze_license
        unfreezeLicense => unfreeze_license
        wipeLicense => wipe_license
        setRoleToContract => set_role_to_contract
        getTokenId => token_id
        getFixedAttributes => fixed_attributes
    )
}

multiversx_sc_wasm_adapter::async_callback! { mx_contracts_rs }
