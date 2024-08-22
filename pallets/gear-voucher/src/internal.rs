// This file is part of Gear.

// Copyright (C) 2021-2024 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use common::{
    storage::{Counter, CounterImpl, Mailbox},
    Origin,
};
use frame_system::pallet_prelude::BlockNumberFor;
use gear_core::ids::{self, CodeId, MessageId, ProgramId};
use sp_std::collections::btree_set::BTreeSet;

impl<T: Config> crate::Call<T>
where
    T::AccountId: Origin,
{
    /// Returns account id that pays for gas purchase and transaction fee
    /// for processing this ['pallet_gear_voucher::Call'], if:
    ///
    /// * Call is [`Self::call`]:
    ///     * Voucher with the given voucher id exists;
    ///     * Caller is eligible to use the voucher;
    ///     * The voucher is not expired;
    ///     * For messaging calls: The destination program of the given prepaid
    ///                            call can be determined;
    ///     * For messaging calls: The voucher destinations limitations accept
    ///                            determined destination;
    ///     * For codes uploading: The voucher allows code uploading.
    ///
    /// Returns [`None`] for other cases.
    pub fn get_sponsor(&self, caller: AccountIdOf<T>) -> Option<AccountIdOf<T>> {
        match self {
            Self::call {
                voucher_id,
                call: prepaid_call,
            } => Pallet::<T>::validate_prepaid(caller, *voucher_id, prepaid_call)
                .map(|_| (*voucher_id).cast())
                .ok(),

            _ => None,
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Queries a voucher and asserts its validity.
    pub fn get_active_voucher(
        origin: AccountIdOf<T>,
        voucher_id: VoucherId,
    ) -> Result<VoucherInfo<AccountIdOf<T>, BlockNumberFor<T>>, Error<T>> {
        let voucher =
            Vouchers::<T>::get(origin.clone(), voucher_id).ok_or(Error::<T>::InexistentVoucher)?;

        ensure!(
            <frame_system::Pallet<T>>::block_number() < voucher.expiry,
            Error::<T>::VoucherExpired
        );

        Ok(voucher)
    }

    /// Validate prepaid call with related params of voucher: origin, expiration.
    pub fn validate_prepaid(
        origin: AccountIdOf<T>,
        voucher_id: VoucherId,
        call: &PrepaidCall<BalanceOf<T>>,
    ) -> Result<(), Error<T>> {
        let voucher = Self::get_active_voucher(origin.clone(), voucher_id)?;

        match call {
            PrepaidCall::DeclineVoucher => (),
            PrepaidCall::UploadCode { .. } => {
                ensure!(
                    voucher.permissions.code_uploading,
                    Error::<T>::CodeUploadingDisabled
                )
            }
            PrepaidCall::SendMessage { .. } | PrepaidCall::SendReply { .. } => {
                if let Some(ref programs) = voucher.permissions.programs {
                    let destination = Self::prepaid_call_destination(&origin, call)
                        .ok_or(Error::<T>::UnknownDestination)?;

                    ensure!(
                        programs.contains(&destination),
                        Error::<T>::InappropriateDestination
                    );
                }
            }
            PrepaidCall::CreateProgram { code_id, .. } => {
                if let Some(code_ids) = voucher.permissions.code_ids {
                    ensure!(code_ids.contains(code_id), Error::<T>::InappropriateCodeId);
                }
            }
        }

        Ok(())
    }

    /// Return destination program of the [`PrepaidCall`], if exists.
    pub fn prepaid_call_destination(
        who: &T::AccountId,
        call: &PrepaidCall<BalanceOf<T>>,
    ) -> Option<ProgramId> {
        match call {
            PrepaidCall::SendMessage { destination, .. } => Some(*destination),
            PrepaidCall::SendReply { reply_to_id, .. } => {
                T::Mailbox::peek(who, reply_to_id).map(|stored_message| stored_message.source())
            }
            PrepaidCall::UploadCode { .. }
            | PrepaidCall::DeclineVoucher
            | PrepaidCall::CreateProgram { .. } => None,
        }
    }
}

/// Trait for processing prepaid calls by any implementor.
pub trait PrepaidCallsDispatcher {
    type AccountId;
    type Balance;

    /// Returns weight of processing for call.
    fn weight(call: &PrepaidCall<Self::Balance>) -> Weight;

    /// Processes prepaid call with specific sponsor from origins address.
    fn dispatch(
        account_id: Self::AccountId,
        sponsor_id: Self::AccountId,
        voucher_id: VoucherId,
        call: PrepaidCall<Self::Balance>,
    ) -> DispatchResultWithPostInfo;
}

/// Voucher identifier.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Eq,
    derive_more::From,
    derive_more::AsRef,
    TypeInfo,
    Encode,
    Decode,
    MaxEncodedLen,
)]
pub struct VoucherId([u8; 32]);

impl VoucherId {
    pub fn generate<T: Config>() -> Self {
        const SALT: &[u8] = b"voucher";

        CounterImpl::<u64, IssuedWrap<T>>::increase();
        let nonce = CounterImpl::<u64, IssuedWrap<T>>::get();

        ids::hash_of_array([SALT, &nonce.to_le_bytes()]).into()
    }
}

impl Origin for VoucherId {
    fn into_origin(self) -> H256 {
        self.0.into()
    }

    fn from_origin(val: H256) -> Self {
        Self(val.to_fixed_bytes())
    }
}

/// Type containing all data about voucher.
#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct VoucherInfo<AccountId, BlockNumber> {
    /// Owner of the voucher.
    /// May be different to original issuer.
    /// Owner manages and claims back remaining balance of the voucher.
    pub owner: AccountId,
    /// The block number at and after which voucher couldn't be used and
    /// can be revoked by owner.
    pub expiry: BlockNumber,
    /// Set of CodeId this voucher could be used to create program.
    /// In case of [`None`] means any uploaded code.
    pub permissions: VoucherPermissions,
}

impl<AccountId, BlockNumber> VoucherInfo<AccountId, BlockNumber> {
    pub fn contains(&self, program_id: ProgramId) -> bool {
        self.permissions
            .programs
            .as_ref()
            .map_or(true, |v| v.contains(&program_id))
    }
}

/// Prepaid call to be executed on-chain.
#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrepaidCall<Balance> {
    SendMessage {
        destination: ProgramId,
        payload: Vec<u8>,
        gas_limit: u64,
        value: Balance,
        keep_alive: bool,
    },
    SendReply {
        reply_to_id: MessageId,
        payload: Vec<u8>,
        gas_limit: u64,
        value: Balance,
        keep_alive: bool,
    },
    UploadCode {
        code: Vec<u8>,
    },
    DeclineVoucher,
    CreateProgram {
        code_id: CodeId,
        salt: Vec<u8>,
        payload: Vec<u8>,
        gas_limit: u64,
        value: Balance,
        keep_alive: bool,
    },
}

/// Voucher Permissions:
/// * programs: pool of programs spender can interact with,
///             if None - means any program,
///             limited by Config param;
/// * code_uploading:
///             allow voucher to be used as payer for `upload_code`
///             transactions fee;
/// * code_ids: pool of code identifiers spender can create program from,
///             if None - means any code,
///             limited by Config param;
#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub struct VoucherPermissions {
    /// Set of programs this voucher could be used to interact with.
    /// In case of [`None`] means any gear program.
    pub programs: Option<BTreeSet<ProgramId>>,
    /// Flag if this voucher's covers uploading codes as prepaid call.
    pub code_uploading: bool,
    /// Set of CodeId this voucher could be used to create program.
    /// In case of [`None`] means any uploaded code.
    pub code_ids: Option<BTreeSet<CodeId>>,
}

impl VoucherPermissions {
    pub const fn none() -> Self {
        Self {
            programs: Some(BTreeSet::new()),
            code_uploading: false,
            code_ids: Some(BTreeSet::new()),
        }
    }

    pub const fn all() -> Self {
        Self {
            programs: None,
            code_uploading: true,
            code_ids: None,
        }
    }

    pub fn allow_code_uploading(self, code_uploading: bool) -> Self {
        Self {
            code_uploading,
            ..self
        }
    }

    pub fn allow_programs(self, programs: Option<BTreeSet<ProgramId>>) -> Self {
        Self { programs, ..self }
    }

    pub fn allow_code_ids(self, code_ids: Option<BTreeSet<CodeId>>) -> Self {
        Self { code_ids, ..self }
    }

    /// Extend permissions
    ///
    /// Returns `true` if permissions extended
    pub fn extend<T: Config>(
        &mut self,
        extend: VoucherPermissionsExtend,
    ) -> Result<bool, Error<T>> {
        // Flag if permissions needs update in storage.
        let mut updated = false;

        // Flattening code uploading.
        let code_uploading = extend.code_uploading.filter(|v| *v != self.code_uploading);
        // Optionally enabling code uploading.
        if let Some(code_uploading) = code_uploading {
            ensure!(code_uploading, Error::<T>::CodeUploadingEnabled);

            self.code_uploading = true;
            updated = true;
        }

        // Optionally extends whitelisted programs with amount validation.
        match extend.append_programs {
            // Adding given destination set to voucher,
            // if it has destinations limit.
            Some(Some(mut extra_programs)) if self.programs.is_some() => {
                let programs = self.programs.as_mut().expect("Infallible");
                let initial_len = programs.len();

                programs.append(&mut extra_programs);

                ensure!(
                    programs.len() <= T::MaxProgramsAmount::get().into(),
                    Error::<T>::MaxProgramsLimitExceeded
                );

                updated |= programs.len() != initial_len;
            }

            // Extending vouchers to unlimited destinations.
            Some(None) => updated |= self.programs.take().is_some(),

            // Noop.
            _ => (),
        }

        // Optionally extends whitelisted code_ids.
        match extend.append_code_ids {
            // Adding given destination set to voucher,
            // if it has destinations limit.
            Some(Some(mut extra_code_ids)) if self.code_ids.is_some() => {
                let code_ids = self.code_ids.as_mut().expect("Infallible; qed");
                let initial_len = code_ids.len();

                code_ids.append(&mut extra_code_ids);

                ensure!(
                    code_ids.len() <= T::MaxCodeIdsAmount::get().into(),
                    Error::<T>::MaxCodeIdsLimitExceeded
                );

                updated |= code_ids.len() != initial_len;
            }

            // Extending vouchers to any CodeId.
            Some(None) => updated |= self.code_ids.take().is_some(),

            // Noop.
            _ => (),
        }

        // Return updated flag
        Ok(updated)
    }
}

impl Default for VoucherPermissions {
    /// Default permissions don't allow anything
    fn default() -> Self {
        Self::none()
    }
}

/// Voucher Permissions Extend
/// * append_programs:  optionally extends pool of programs by
///                     `Some(programs_set)` passed or allows
///                     it to interact with any program by
///                     `None` passed;
/// * code_uploading:   optionally allows voucher to be used to pay
///                     fees for `upload_code` extrinsics;
/// * append_code_ids:  optionally extends pool of code identifiers
///                     `Some(code_ids)` passed or allows
///                     it to interact with any program by
///                     `None` passed;
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub struct VoucherPermissionsExtend {
    pub append_programs: Option<Option<BTreeSet<ProgramId>>>,
    pub code_uploading: Option<bool>,
    pub append_code_ids: Option<Option<BTreeSet<CodeId>>>,
}

impl From<Option<VoucherPermissionsExtend>> for VoucherPermissionsExtend {
    fn from(value: Option<VoucherPermissionsExtend>) -> Self {
        value.unwrap_or_default()
    }
}
