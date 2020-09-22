use crate::Nonce;

use crate::{
    tx::{ChangePubKey, Close, Transfer, TxEthSignature, TxHash, Withdraw},
    CloseOp, TokenLike, TransferOp, TxFeeTypes, WithdrawOp,
};
use num::BigUint;
use parity_crypto::digest::sha256;

use crate::operations::ChangePubKeyOp;
use serde::{Deserialize, Serialize};
use zksync_basic_types::Address;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EthSignData {
    pub signature: TxEthSignature,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedFranklinTx {
    pub tx: FranklinTx,
    pub eth_sign_data: Option<EthSignData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FranklinTx {
    Transfer(Box<Transfer>),
    Withdraw(Box<Withdraw>),
    Close(Box<Close>),
    ChangePubKey(Box<ChangePubKey>),
}

impl From<FranklinTx> for SignedFranklinTx {
    fn from(tx: FranklinTx) -> Self {
        Self {
            tx,
            eth_sign_data: None,
        }
    }
}

impl std::ops::Deref for SignedFranklinTx {
    type Target = FranklinTx;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl FranklinTx {
    pub fn hash(&self) -> TxHash {
        let bytes = match self {
            FranklinTx::Transfer(tx) => tx.get_bytes(),
            FranklinTx::Withdraw(tx) => tx.get_bytes(),
            FranklinTx::Close(tx) => tx.get_bytes(),
            FranklinTx::ChangePubKey(tx) => tx.get_bytes(),
        };

        let hash = sha256(&bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&hash);
        TxHash { data: out }
    }

    pub fn account(&self) -> Address {
        match self {
            FranklinTx::Transfer(tx) => tx.from,
            FranklinTx::Withdraw(tx) => tx.from,
            FranklinTx::Close(tx) => tx.account,
            FranklinTx::ChangePubKey(tx) => tx.account,
        }
    }

    pub fn nonce(&self) -> Nonce {
        match self {
            FranklinTx::Transfer(tx) => tx.nonce,
            FranklinTx::Withdraw(tx) => tx.nonce,
            FranklinTx::Close(tx) => tx.nonce,
            FranklinTx::ChangePubKey(tx) => tx.nonce,
        }
    }

    pub fn check_correctness(&mut self) -> bool {
        match self {
            FranklinTx::Transfer(tx) => tx.check_correctness(),
            FranklinTx::Withdraw(tx) => tx.check_correctness(),
            FranklinTx::Close(tx) => tx.check_correctness(),
            FranklinTx::ChangePubKey(tx) => tx.check_correctness(),
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match self {
            FranklinTx::Transfer(tx) => tx.get_bytes(),
            FranklinTx::Withdraw(tx) => tx.get_bytes(),
            FranklinTx::Close(tx) => tx.get_bytes(),
            FranklinTx::ChangePubKey(tx) => tx.get_bytes(),
        }
    }

    pub fn min_chunks(&self) -> usize {
        match self {
            FranklinTx::Transfer(_) => TransferOp::CHUNKS,
            FranklinTx::Withdraw(_) => WithdrawOp::CHUNKS,
            FranklinTx::Close(_) => CloseOp::CHUNKS,
            FranklinTx::ChangePubKey(_) => ChangePubKeyOp::CHUNKS,
        }
    }

    pub fn is_withdraw(&self) -> bool {
        match self {
            FranklinTx::Withdraw(_) => true,
            _ => false,
        }
    }

    pub fn is_close(&self) -> bool {
        match self {
            FranklinTx::Close(_) => true,
            _ => false,
        }
    }

    pub fn get_fee_info(&self) -> Option<(TxFeeTypes, TokenLike, Address, BigUint)> {
        match self {
            FranklinTx::Withdraw(withdraw) => {
                let fee_type = if withdraw.fast {
                    TxFeeTypes::FastWithdraw
                } else {
                    TxFeeTypes::Withdraw
                };

                Some((
                    fee_type,
                    TokenLike::Id(withdraw.token),
                    withdraw.to,
                    withdraw.fee.clone(),
                ))
            }
            FranklinTx::Transfer(transfer) => Some((
                TxFeeTypes::Transfer,
                TokenLike::Id(transfer.token),
                transfer.to,
                transfer.fee.clone(),
            )),
            _ => None,
        }
    }
}
