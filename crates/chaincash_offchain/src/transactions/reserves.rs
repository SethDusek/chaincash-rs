use crate::boxes::ReserveBoxSpec;

use super::{TransactionError, TxContext};
use ergo_lib::chain::ergo_box::box_builder::ErgoBoxCandidateBuilder;
use ergo_lib::chain::transaction::unsigned::UnsignedTransaction;
use ergo_lib::chain::transaction::Transaction;
use ergo_lib::ergo_chain_types::EcPoint;
use ergo_lib::ergotree_ir::chain::address::NetworkAddress;
use ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox;
use ergo_lib::ergotree_ir::chain::{ergo_box::NonMandatoryRegisterId, token::Token};
use ergo_lib::ergotree_ir::ergo_tree::ErgoTree;
use ergo_lib::wallet::signing::ErgoTransaction;
use ergo_lib::wallet::{box_selector::BoxSelection, tx_builder::TxBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MintReserveRequest {
    pub public_key_hex: String,
    pub amount: u64,
}

pub struct MintReserveResponse<T: ErgoTransaction> {
    /// Reserve Box
    pub reserve_box: ReserveBoxSpec,
    /// Unsigned transaction that creates reserve box and mints reserve NFT
    pub transaction: T,
}

pub type SignedMintReserveResponse = MintReserveResponse<Transaction>;

pub fn mint_reserve_transaction(
    request: MintReserveRequest,
    reserve_tree: ErgoTree,
    inputs: BoxSelection<ErgoBox>,
    context: TxContext,
) -> Result<MintReserveResponse<UnsignedTransaction>, TransactionError> {
    let pk = EcPoint::try_from(request.public_key_hex).map_err(TransactionError::Parsing)?;
    let mut reserve_box_builder = ErgoBoxCandidateBuilder::new(
        request.amount.try_into()?,
        reserve_tree,
        context.current_height,
    );
    let nft_id = inputs
        .boxes
        .get(0)
        .ok_or_else(|| {
            TransactionError::MissingBox(
                "mint_reserve_transaction: failed to find input box required to mint nft"
                    .to_string(),
            )
        })?
        .box_id();
    let nft = Token {
        token_id: nft_id.into(),
        amount: 1.try_into()?,
    };
    reserve_box_builder.add_token(nft);
    reserve_box_builder.set_register_value(NonMandatoryRegisterId::R4, pk.into());

    let unsigned_transaction = TxBuilder::new(
        inputs,
        vec![reserve_box_builder.build()?],
        context.current_height,
        context.fee.try_into()?,
        NetworkAddress::try_from(context.change_address)?.address(),
    )
    .build()?;

    Ok(MintReserveResponse {
        reserve_box: unsigned_transaction.outputs().first().try_into().unwrap(), // TODO: is unwrap() a good idea here? Since reserve box is built by us any error in TryFrom is a bug in chaincash-rs and not user error
        transaction: unsigned_transaction,
    })
}
