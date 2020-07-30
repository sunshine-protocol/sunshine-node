use codec::{
    Codec,
    Decode,
    Encode,
};
use frame_support::Parameter;
use libipld::{
    cbor::DagCborCodec,
    codec::{
        Decode as DagEncode,
        Encode as DagDecode,
    },
};
use sp_runtime::traits::{
    AtLeast32Bit,
    MaybeSerializeDeserialize,
    Member,
    Zero,
};
use std::fmt::Debug;
use substrate_subxt::{
    module,
    sp_runtime,
    system::{
        System,
        SystemEventsDecoder,
    },
    Call,
    Event,
};

pub type BalanceOf<T> = <T as Bounty>::Currency;

#[module]
pub trait Bounty: System {
    /// Cid type
    type IpfsReference: Parameter + Member + Default;
    /// Currency type
    type Currency: Parameter
        + Member
        + AtLeast32Bit
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + PartialOrd
        + PartialEq
        + Zero;

    type BountyId: Parameter
        + Member
        + AtLeast32Bit
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + PartialOrd
        + PartialEq
        + Zero;

    /// The shape of bounty postings
    type BountyPost: 'static
        + Codec
        + Default
        + Clone
        + DagEncode<DagCborCodec>
        + DagDecode<DagCborCodec>
        + Send
        + Sync;

    type SubmissionId: Parameter
        + Member
        + AtLeast32Bit
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + PartialOrd
        + PartialEq
        + Zero;

    /// The shape of bounty submission
    type BountySubmission: 'static
        + Codec
        + Default
        + Clone
        + DagEncode<DagCborCodec>
        + DagDecode<DagCborCodec>
        + Send
        + Sync;
}

// ~~ (Calls, Events) ~~

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct PostBountyCall<T: Bounty> {
    pub info: T::IpfsReference,
    pub amount: BalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct BountyPostedEvent<T: Bounty> {
    pub depositer: <T as System>::AccountId,
    pub amount: BalanceOf<T>,
    pub id: T::BountyId,
    pub description: T::IpfsReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct ContributeToBountyCall<T: Bounty> {
    pub bounty_id: T::BountyId,
    pub amount: BalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct BountyRaiseContributionEvent<T: Bounty> {
    pub contributor: <T as System>::AccountId,
    pub new_amount: BalanceOf<T>,
    pub bounty_id: T::BountyId,
    pub total: BalanceOf<T>,
    pub bounty_ref: T::IpfsReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct SubmitForBountyCall<T: Bounty> {
    pub bounty_id: T::BountyId,
    pub submission_ref: T::IpfsReference,
    pub amount: BalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct BountySubmissionPostedEvent<T: Bounty> {
    pub submitter: <T as System>::AccountId,
    pub bounty_id: T::BountyId,
    pub amount: BalanceOf<T>,
    pub id: T::SubmissionId,
    pub bounty_ref: T::IpfsReference,
    pub submission_ref: T::IpfsReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct ApproveBountySubmissionCall<T: Bounty> {
    pub submission_id: T::SubmissionId,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct BountyPaymentExecutedEvent<T: Bounty> {
    pub bounty_id: T::BountyId,
    pub new_total: BalanceOf<T>,
    pub submission_id: T::SubmissionId,
    pub amount: BalanceOf<T>,
    pub submitter: <T as System>::AccountId,
    pub bounty_ref: T::IpfsReference,
    pub submission_ref: T::IpfsReference,
}
