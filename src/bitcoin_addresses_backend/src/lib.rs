mod schnorr_api;

use bitcoin::hashes::{sha256, Hash};
use bitcoin::key::Secp256k1;
use bitcoin::script::{Builder, ScriptBuf};
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use candid::{CandidType, Deserialize};

use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
};

use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

use schnorr_api::schnorr_public_key;

use std::cell::RefCell;

thread_local! {

    // The key name. Used for ECDSA and Schnorr keys.
    static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));
}

#[derive(CandidType, Deserialize, Debug)]
enum BitcoinAddress {
    P2pkh,  // Pay to Public Key Hash
    P2sh,   // Pay to Script Hash
    P2wpkh, // Pay to Witness Public Key Hash
    P2wsh,  // Pay to Witness Script Hash
    P2tr,   // Taproot address
}

#[derive(CandidType, Deserialize, Debug)]
enum Key {
    Dfx,
    Test,
    Prod,
}

impl Key {
    fn as_str(&self) -> &'static str {
        match self {
            Key::Dfx => "dfx_test_key",
            Key::Test => "test_key_1",
            Key::Prod => "key_1",
        }
    }
}

#[ic_cdk::init]
fn init(key: Key) {

    ic_cdk::println!("Setting key to {:?}", key.as_str());

    KEY_NAME.with(|name| {
        name.replace(key.as_str().to_string());
    });
}

#[ic_cdk::update]
async fn generate_address(
    network: BitcoinNetwork,
    address_type: BitcoinAddress,
    // Use principal of caller as derivation path to generate the address
    use_caller: bool,
) -> Result<String, String> {
    let network = match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    };

    let derivation_path = if use_caller {
        vec![ic_cdk::caller().as_slice().to_vec()]
    } else {
        vec![]
    };

  
    let address = match address_type {
        BitcoinAddress::P2pkh => get_p2pkh_address(network, derivation_path).await,
        BitcoinAddress::P2sh => get_p2sh_address(network, derivation_path).await,
        BitcoinAddress::P2wpkh => get_p2wpkh_address(network, derivation_path).await,
        BitcoinAddress::P2wsh => get_p2wsh_address(network, derivation_path).await,
        BitcoinAddress::P2tr => get_p2tr_address(network, derivation_path).await,
    };

    Ok(address.to_string())
}

async fn get_ecdsa_public_key(derivation_path: Vec<Vec<u8>>) -> CompressedPublicKey {

    KEY_NAME.with(|name| {
        ic_cdk::println!("Key name: {:?}", name.borrow());
    });

    let arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: KEY_NAME.with(|name| name.borrow().clone()),
        },
    };
    let (response,): (EcdsaPublicKeyResponse,) = ecdsa_public_key(arg)
        .await
        .expect("Failed to get public key");
    CompressedPublicKey::from_slice(&response.public_key).expect("Invalid public key")
}

async fn get_schnnorr_public_key(derivation_path: Vec<Vec<u8>>) -> PublicKey {
    let key_name = KEY_NAME.with(|name| name.borrow().clone());
    let raw_schnorr_public_key = schnorr_public_key(key_name, derivation_path).await;
    PublicKey::from_slice(&raw_schnorr_public_key)
        .unwrap()
        .into()
}

async fn get_p2pkh_address(network: Network, derivation_path: Vec<Vec<u8>>) -> Address {
    let public_key = get_ecdsa_public_key(derivation_path).await;
    Address::p2pkh(&public_key, network)
}

async fn get_p2sh_address(network: Network, derivation_path: Vec<Vec<u8>>) -> Address {
    let public_key = get_ecdsa_public_key(derivation_path).await;

    // Build the P2PKH scriptPubKey
    let script_pubkey = create_p2pkh_script(&public_key);

    Address::p2sh(&script_pubkey, network).expect("Failed to create P2SH address")
}

async fn get_p2wsh_address(network: Network, derivation_path: Vec<Vec<u8>>) -> Address {
    let public_key = get_ecdsa_public_key(derivation_path).await;

    // Build the P2PKH scriptPubKey
    let script_pubkey = create_p2pkh_script(&public_key);

    Address::p2wsh(&script_pubkey, network)
}

async fn get_p2wpkh_address(network: Network, derivation_path: Vec<Vec<u8>>) -> Address {
    let public_key = get_ecdsa_public_key(derivation_path).await;
    Address::p2wpkh(&public_key, network)
}

async fn get_p2tr_address(network: Network, derivation_path: Vec<Vec<u8>>) -> Address {
    let secp256k1 = Secp256k1::new();

    let public_key = get_schnnorr_public_key(derivation_path).await;

    Address::p2tr(&secp256k1, public_key.into(), None, network)
}

fn create_p2pkh_script(public_key: &CompressedPublicKey) -> ScriptBuf {
    use bitcoin::blockdata::opcodes::all as opcode;

    let pubkey_bytes = public_key.to_bytes();

    Builder::new()
        .push_opcode(opcode::OP_DUP)
        .push_opcode(opcode::OP_HASH160)
        .push_slice(&sha256::Hash::hash(&pubkey_bytes).as_byte_array())
        .push_opcode(opcode::OP_EQUALVERIFY)
        .push_opcode(opcode::OP_CHECKSIG)
        .into_script()
}

ic_cdk::export_candid!();
