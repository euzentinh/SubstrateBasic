#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
use frame_support::inherent::Vec;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::{*, ValueQuery, StorageMap}, Blake2_128Concat};
	use frame_system::{pallet_prelude::*};
	use frame_support::inherent::Vec;

	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}

	pub type Id = u32;

	// pub type AccountId;
	#[derive(TypeInfo, Encode, Decode)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self{
			Gender::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	// #[pallet::getter(fn kitty_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyId<T:Config> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	pub(super) type Kitties<T:Config> = StorageMap<_,Blake2_128Concat,Id, Kitty<T>, OptionQuery,>;

	#[pallet::storage]
	pub(super) type Owners<T:Config> = StorageMap<_,Blake2_128Concat,T::AccountId, Vec<Vec<u8>>,OptionQuery,>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		SomethingRemoved(T::AccountId),
		NewKittyCreated(T::AccountId, Vec<u8>),
		KittyOwnerChanged(T::AccountId, T::AccountId, Vec<u8>),
		KittyOwnerChangeFailed(T::AccountId, T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8> , price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let gender = Self::check_gender(dna.clone())?;

			let new_kitty: Kitty<T> = Kitty {
				dna: dna.clone(),
				owner: who.clone(),
				price: price,
				gender: gender,
			};

			let current_id = <KittyId<T>>::get();
			let new_id = current_id + 1;

			let owner = <Owners<T>>::get(who.clone());
			let mut new_owner = match owner {
				None => Vec::new(),
				_ => owner.unwrap(),
			};
			new_owner.push(dna.clone());
			// Update storage.
			KittyId::<T>::put(new_id);
			<Kitties<T>>::insert(new_id, &new_kitty);
			<Owners<T>>::insert(who.clone(), new_owner);
			// Emit an event.
			Self::deposit_event(Event::NewKittyCreated(who, dna.clone()));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(origin: OriginFor<T>, dna: Vec<u8> , owner_id: T::AccountId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let from_owner = <Owners<T>>::get(who.clone());
			let to_owner = <Owners<T>>::get(owner_id.clone());
			let mut new_from_owner = match from_owner {
				None => Vec::new(),
				_ => from_owner.unwrap(),
			};
			let mut new_to_owner = match to_owner {
				None => Vec::new(),
				_ => to_owner.unwrap(),
			};
			if let Some(pos) = new_from_owner.iter().position(|x| *x == dna) {
				new_from_owner.swap_remove(pos);
				new_to_owner.push(dna.clone());
				<Owners<T>>::insert(who.clone(), new_from_owner);
				<Owners<T>>::insert(owner_id.clone(), new_to_owner);
				Self::deposit_event(Event::KittyOwnerChanged(who.clone(), owner_id, dna.clone()));
			}
			else {
				Self::deposit_event(Event::KittyOwnerChangeFailed(who.clone(), owner_id, dna.clone()));
			}
			Ok(())
		}
	}
}

// helper function
impl<T> Pallet<T> {
	fn check_gender(dna: Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Male;
		if dna.len() % 2 !=0 {
			res = Gender::Female;
		}
		Ok(res)
	}
}