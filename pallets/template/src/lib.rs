#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_support::{dispatch::DispatchResult,
    pallet_prelude::*, traits::Currency};
use frame_support::sp_runtime::traits::Convert;
use frame_system::pallet_prelude::*;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use sp_runtime::traits::Zero;
use sp_runtime::traits::AtLeast32BitUnsigned;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AuthInfo<BlockNumber, AccountId> 
    where BlockNumber: Parameter + AtLeast32BitUnsigned{
    pub hash : Vec<u8>, //作品id
    pub accountld : AccountId,
    pub blocknumber : BlockNumber,
    pub description: Vec<u8>,
    pub orgcode : Vec<u8>, //公司id
}

impl<BlockNumber, AccountId> AuthInfo<BlockNumber, AccountId> 
    where BlockNumber: Parameter + AtLeast32BitUnsigned{
    pub fn new( hash: Vec<u8>,
                accountld: AccountId,
                blocknumber: BlockNumber,
                description: Vec<u8>,
                orgcode: Vec<u8>) -> Self {
        Self{
            hash,
            accountld,
            blocknumber,
            description,
            orgcode,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
pub struct OrgInfo {
    pub code : Vec<u8>,
    pub name : Vec<u8>,
    pub status: bool,
}

impl OrgInfo {
    pub fn new( code: Vec<u8>,
                name: Vec<u8>,
                status: bool) -> Self {
        Self {
            code,
            name,
            status,
        }
    }
}

  #[frame_support::pallet]
  pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::{pallet_prelude::*, Account};

     /* Placeholder for defining custom types. */

      // TODO: Update the `config` block below
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // currency to pay fees and hold balances
        type Currency: Currency<Self::AccountId>;
        // amount converted to numbers
        type BalanceToNumber: Convert<BalanceOf<Self>, u128>;
    }

      // The struct on which we build all of our Pallet logic.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);
	
    //AuthRight information, to quickiy locate AuthRight
    #[pallet::storage]
    #[pallet::getter(fn authright)]
    pub(super) type AuthRight<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        T::AccountId,
        OptionQuery,
    >;

    //Details of the copyright
    #[pallet::storage]
	#[pallet::getter(fn authdetail)]
	pub type AuthDetail<T : Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        Vec<u8>, 
        AuthInfo<T::BlockNumber, T::AccountId>,
        OptionQuery,
    >;

    //The information of organization
    #[pallet::storage]
	#[pallet::getter(fn org)]
	pub type Org<T : Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        Vec<u8>, 
        OrgInfo,
        OptionQuery,
    >;
     
      // TODO: Update the `event` block below
      #[pallet::event]
      #[pallet::generate_deposit(pub(super) fn deposit_event)]
      pub enum Event<T: Config> {
        // (accountid, hash, orgcode)
        AuthRightSuccessed(T::AccountId, Vec<u8>, Vec<u8>),
        // (orgCode, orgName) )
        OrgRegSuccess(T::AccountId, Vec<u8>, Vec<u8>),
        // orgApprove(orgCode, status)
        OrgApproveSuccess(T::AccountId, Vec<u8>, bool),
      }

      // TODO: Update the `error` block below
      #[pallet::error]
        pub enum Error<T> {
            NoSuchOrg,

            OrgAlreadyExist,

            StatusNotAllow,

            HashAlreadyExist,
        }

      // TODO: add #[pallet::storage] block

      // TODO: Update the `call` block below
      #[pallet::call]
      impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn orgReg(
            origin: OriginFor<T>,
            orgcode: Vec<u8>,
            orgname: Vec<u8>,
        ) -> DispatchResult {
            
            let who = ensure_signed(origin)?;
            //This orgnation is already exist
            ensure!(!Org::<T>::contains_key(orgcode.clone()), Error::<T>::OrgAlreadyExist);

            //crate the new ortInfo struct, and save in Org
            let new_org_info = OrgInfo::new(
                orgcode.clone(),
                orgname.clone(),
                false,
            );
            Org::<T>::insert(orgcode.clone(), new_org_info);

            //send the success event 
            Self::deposit_event(Event::OrgRegSuccess(who, orgcode.clone(), orgname.clone()));

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn orgApprove(
            origin: OriginFor<T>,
            orgcode: Vec<u8>,
            status : bool,
        ) -> DispatchResult {
            
            ensure_root(origin.clone())?;
            let who = ensure_signed(origin)?;

            //This organzation not exist
            ensure!(Org::<T>::contains_key(orgcode.clone()), Error::<T>::NoSuchOrg);

            //Get the old organzation,and change it's status            
            let mut orginfo = Org::<T>::get(orgcode.clone()).unwrap();
            orginfo.status = status;

            //Save the new status of organzation into Org
            Org::<T>::insert(orgcode.clone(), orginfo);
            
            Self::deposit_event(Event::OrgApproveSuccess(who.clone(), orgcode.clone(), status));

            Ok(())
        }

        #[frame_support::transactional]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authRight(
            origin: OriginFor<T>,
            hash: Vec<u8>,
            description: Vec<u8>,
            orgcode : Vec<u8>,
        ) -> DispatchResult {
            
            let who = ensure_signed(origin)?;

            //This hashid has already exist, return Error
            ensure!(!AuthRight::<T>::contains_key(hash.clone()), Error::<T>::HashAlreadyExist);
            
            //This organization has't exist in the chain, return Error
            ensure!(Org::<T>::contains_key(orgcode.clone()), Error::<T>::NoSuchOrg);
            
            let org = Org::<T>::get(orgcode.clone()).unwrap();

            //This organization's status not allow to define rights
            ensure!(org.status, Error::<T>::StatusNotAllow);

            //Resave the org
            Org::<T>::insert(orgcode.clone(), org.clone());
            
            // get the current block height
            let block_number = <frame_system::Pallet<T>>::block_number();

            let mut new_auth_info = AuthInfo::new(
                hash.clone(),
                who.clone(),
                block_number,
                description.clone(),
                orgcode.clone(),
            );

            //Save the message into AuthRight and AuthDetail 
            AuthRight::<T>::insert(hash.clone(), who.clone());
            AuthDetail::<T>::insert(hash.clone(), new_auth_info);

            //Send the success event 
            Self::deposit_event(Event::<T>::AuthRightSuccessed(who.clone(), hash.clone(), orgcode.clone()));

            Ok(())
        }
    }
}