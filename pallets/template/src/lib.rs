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
        Vec<u8>,
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
        /*
        orgReg(orgCode, orgName) 
        //作用：把公司注册到链上
        1.查询该公司的 code是否已经在链上存储了
            如果已经存储了，直接return Ok(())
        2.查询为空
            2.1创建新的orginfo结构体
            2.2 Code = orgCode, Name = orgName, Status = false
            2.3 将该结构体插入
            2.4 发送注册成功事件， return Ok(())
        */
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn orgReg(
            origin: OriginFor<T>,
            orgcode: Vec<u8>,
            orgname: Vec<u8>,
        ) -> DispatchResult {
            
            let who = ensure_signed(origin)?;
            //This orgnation is already exist
            ensure!(!Org::<T>::contains_key(orgcode.clone()), Error::<T>::OrgAlreadyExist);

            let new_org_info = OrgInfo::new(
                orgcode.clone(),
                orgname.clone(),
                false,
            );
            
            Org::<T>::insert(orgcode.clone(), new_org_info);
            Self::deposit_event(Event::OrgRegSuccess(who, orgcode.clone(), orgname.clone()));

            Ok(())
        }

        /*
        orgApprove(orgCode, status)
        //作用：管理员将公司状态 切换为 true
        1. 查询 orgCode是否在链上
            查询失败，发送失败的event return Ok(())
        2. 查询成功
            2.1 取出查询的公司的结构体，将该公司的status设置为 status
            2.2 发送 orgApprove事件，return Ok(()) */
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
            
            let mut orginfo = Org::<T>::get(orgcode.clone()).unwrap();
            orginfo.status = status;

            Org::<T>::insert(orgcode.clone(), orginfo);
            Self::deposit_event(Event::OrgApproveSuccess(who.clone(), orgcode.clone(), status));

            Ok(())
        }
    }
}