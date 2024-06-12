#![allow(dead_code, clippy::new_without_default)]

use abi::{AlloyProgram, AlloyRouter};
use alloy::{
    consensus::{SidecarBuilder, SignableTransaction, SimpleCoder},
    network::{Ethereum, EthereumSigner, TxSigner},
    primitives::{Address, Bytes, ChainId, Signature, B256},
    providers::{
        fillers::{FillProvider, JoinFill, RecommendedFiller, SignerFiller},
        ProviderBuilder, RootProvider,
    },
    pubsub::PubSubFrontend,
    rpc::client::WsConnect,
    signers::{
        self as alloy_signer, sign_transaction_with_chain_id, Error as SignerError,
        Result as SignerResult, Signer, SignerSync,
    },
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use gear_core::code::{Code, CodeAndId};
use gear_wasm_instrument::gas_metering::Schedule;
use gprimitives::{ActorId, CodeId, MessageId, H256};
use hypercore_signer::{
    Address as HypercoreAddress, PublicKey, Signature as HypercoreSignature,
    Signer as HypercoreSigner,
};

mod abi;
pub mod event;

type AlloyTransport = PubSubFrontend;
type AlloyProvider = FillProvider<
    JoinFill<RecommendedFiller, SignerFiller<EthereumSigner>>,
    RootProvider<AlloyTransport>,
    AlloyTransport,
    Ethereum,
>;

type AlloyProgramInstance = AlloyProgram::AlloyProgramInstance<AlloyTransport, AlloyProvider>;
type AlloyRouterInstance = AlloyRouter::AlloyRouterInstance<AlloyTransport, AlloyProvider>;

#[derive(Debug, Clone)]
struct Sender {
    signer: HypercoreSigner,
    sender: PublicKey,
    chain_id: Option<ChainId>,
}

impl Sender {
    pub fn new(signer: HypercoreSigner, sender_address: HypercoreAddress) -> Result<Self> {
        let sender = signer
            .get_key_by_addr(sender_address)?
            .ok_or_else(|| anyhow!("no key found for {sender_address}"))?;
        Ok(Self {
            signer,
            sender,
            chain_id: None,
        })
    }
}

#[async_trait]
impl Signer for Sender {
    async fn sign_hash(&self, hash: &B256) -> SignerResult<Signature> {
        self.sign_hash_sync(hash)
    }

    fn address(&self) -> Address {
        self.sender.to_address().0.into()
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

#[async_trait]
impl TxSigner<Signature> for Sender {
    fn address(&self) -> Address {
        self.sender.to_address().0.into()
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> SignerResult<Signature> {
        sign_transaction_with_chain_id!(self, tx, self.sign_hash_sync(&tx.signature_hash()))
    }
}

impl SignerSync for Sender {
    fn sign_hash_sync(&self, hash: &B256) -> SignerResult<Signature> {
        let signature = self
            .signer
            .raw_sign_digest(self.sender, hash.0)
            .map_err(|err| SignerError::Other(err.into()))?;
        Ok(Signature::try_from(&signature.0[..])?)
    }

    fn chain_id_sync(&self) -> Option<ChainId> {
        self.chain_id
    }
}

#[derive(Debug, Clone)]
#[repr(packed)]
pub struct Transition {
    pub actor_id: ActorId,
    pub old_state_hash: H256,
    pub new_state_hash: H256,
}

pub struct Router(AlloyRouterInstance);

impl Router {
    fn new(address: Address, provider: &AlloyProvider) -> Self {
        Self(AlloyRouterInstance::new(address, provider.clone()))
    }

    pub async fn set_program(&self, program: ActorId) -> Result<H256> {
        let builder = self.0.setProgram({
            let mut address = Address::ZERO;
            address.0.copy_from_slice(&program.into_bytes()[12..]);
            address
        });
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn add_validators(&self, validators: Vec<ActorId>) -> Result<H256> {
        let builder = self.0.addValidators(
            validators
                .into_iter()
                .map(|actor_id| {
                    let mut address = Address::ZERO;
                    address.0.copy_from_slice(&actor_id.into_bytes()[12..]);
                    address
                })
                .collect(),
        );
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn remove_validators(&self, validators: Vec<ActorId>) -> Result<H256> {
        let builder = self.0.removeValidators(
            validators
                .into_iter()
                .map(|actor_id| {
                    let mut address = Address::ZERO;
                    address.0.copy_from_slice(&actor_id.into_bytes()[12..]);
                    address
                })
                .collect(),
        );
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn upload_code(&self, code_id: CodeId, blob_tx: H256) -> Result<H256> {
        let builder = self.0.uploadCode(
            B256::new(code_id.into_bytes()),
            B256::new(blob_tx.to_fixed_bytes()),
        );
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn upload_code_with_sidecar(&self, code: &[u8]) -> Result<H256> {
        let schedule = Schedule::default();
        let code = Code::try_new(
            code.to_vec(),
            schedule.instruction_weights.version,
            |module| schedule.rules(module),
            schedule.limits.stack_height,
            schedule.limits.data_segments_amount.into(),
        )
        .map_err(|err| anyhow!("failed to validate code: {err}"))?;
        let (code, code_id) = CodeAndId::new(code).into_parts();

        let builder = self
            .0
            .uploadCode(B256::new(code_id.into_bytes()), B256::ZERO)
            .sidecar(SidecarBuilder::<SimpleCoder>::from_slice(code.original_code()).build()?);
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn create_program(
        &self,
        code_id: CodeId,
        salt: H256,
        init_payload: impl AsRef<[u8]>,
        gas_limit: u64,
        value: u128,
    ) -> Result<H256> {
        let builder = self
            .0
            .createProgram(
                B256::new(code_id.into_bytes()),
                B256::new(salt.to_fixed_bytes()),
                Bytes::copy_from_slice(init_payload.as_ref()),
                gas_limit,
            )
            .value(value.try_into()?);
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn commit_codes(
        &self,
        code_ids: Vec<CodeId>,
        signatures: Vec<HypercoreSignature>,
    ) -> Result<H256> {
        let builder = self.0.commitCodes(
            code_ids
                .into_iter()
                .map(|code_id| B256::new(code_id.into_bytes()))
                .collect(),
            signatures
                .into_iter()
                .map(|signature| Bytes::copy_from_slice(&signature.0))
                .collect(),
        );
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn commit_transitions(
        &self,
        transitions: Vec<Transition>,
        signatures: Vec<HypercoreSignature>,
    ) -> Result<H256> {
        let builder = self.0.commitTransitions(
            transitions
                .into_iter()
                .map(|transition| AlloyRouter::Transition {
                    actorId: {
                        let mut address = Address::ZERO;
                        address
                            .0
                            .copy_from_slice(&transition.actor_id.into_bytes()[12..]);
                        address
                    },
                    oldStateHash: B256::new(transition.old_state_hash.to_fixed_bytes()),
                    newStateHash: B256::new(transition.new_state_hash.to_fixed_bytes()),
                })
                .collect(),
            signatures
                .into_iter()
                .map(|signature| Bytes::copy_from_slice(&signature.0))
                .collect(),
        );
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }
}

pub struct Program(AlloyProgramInstance);

impl Program {
    fn new(address: Address, provider: &AlloyProvider) -> Self {
        Self(AlloyProgramInstance::new(address, provider.clone()))
    }

    pub async fn send_message(
        &self,
        payload: impl AsRef<[u8]>,
        gas_limit: u64,
        value: u128,
    ) -> Result<H256> {
        let builder = self
            .0
            .sendMessage(Bytes::copy_from_slice(payload.as_ref()), gas_limit)
            .value(value.try_into()?);
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn send_reply(
        &self,
        reply_to_id: MessageId,
        payload: impl AsRef<[u8]>,
        gas_limit: u64,
        value: u128,
    ) -> Result<H256> {
        let builder = self
            .0
            .sendReply(
                B256::new(reply_to_id.into_bytes()),
                Bytes::copy_from_slice(payload.as_ref()),
                gas_limit,
            )
            .value(value.try_into()?);
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }

    pub async fn claim_value(&self, message_id: MessageId) -> Result<H256> {
        let builder = self.0.claimValue(B256::new(message_id.into_bytes()));
        let tx = builder.send().await?;
        let receipt = tx.get_receipt().await?;
        Ok(H256(receipt.transaction_hash.0))
    }
}

pub struct HypercoreEthereum {
    router_address: Address,
    provider: AlloyProvider,
}

impl HypercoreEthereum {
    pub async fn new(
        rpc_url: &str,
        router_address: HypercoreAddress,
        signer: HypercoreSigner,
        sender_address: HypercoreAddress,
    ) -> Result<Self> {
        Ok(Self {
            router_address: Address::new(router_address.0),
            provider: ProviderBuilder::new()
                .with_recommended_fillers()
                .signer(EthereumSigner::new(Sender::new(signer, sender_address)?))
                .on_ws(WsConnect::new(rpc_url))
                .await?,
        })
    }

    pub fn router(&self) -> Router {
        Router::new(self.router_address, &self.provider)
    }

    pub fn program(&self, program_address: HypercoreAddress) -> Program {
        Program::new(Address::new(program_address.0), &self.provider)
    }
}
