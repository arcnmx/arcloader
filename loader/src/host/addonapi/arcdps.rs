use nexus::event::arc::{CombatData as NexusCombatData, AgentUpdate as NexusAgentUpdate};
use ::{
	arcdps::{Event, Agent},
	windows_strings::PCSTR,
};
use crate::{
	host::addonapi::NexusHost,
	util::ffi::{cstr_opt, cstr_write, nonnull_const, nonnull_ref},
};
use std::{collections::BTreeMap, ffi::{c_char, c_void, CStr}, mem::{transmute, MaybeUninit}, ptr::{self, NonNull}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};

pub static ARCDPS_CACHE: RwLock<ArcDpsCache> = RwLock::new(ArcDpsCache::new());

pub struct ArcDpsCache {
	account: [c_char; 64],
	pub last_target: Option<ArcDpsAgentUpdate>,
	pub squad_members: BTreeMap<usize, ArcDpsAgentUpdate>,
}

impl ArcDpsCache {
	pub const fn new() -> Self {
		Self {
			account: [0; 64],
			last_target: None,
			squad_members: BTreeMap::new(),
		}
	}

	pub fn account_name(&self) -> Option<NonNull<c_char>> {
		match &self.account[0] {
			0 => None,
			account => Some(nonnull_ref(account)),
		}
	}

	pub fn last_target(&self) -> Option<NonNull<NexusAgentUpdate>> {
		self.last_target.as_ref()
			.and_then(|target| nonnull_const(target.as_nexus_ptr()))
	}

	pub fn set_account_name(&mut self, name: &CStr) {
		let name = name.to_bytes_with_nul();
		let last = unsafe {
			let len = name.len().min(self.account.len());
			ptr::copy_nonoverlapping(name.as_ptr() as *const c_char, self.account.as_mut_ptr(), len);
			let end = self.account.len() - 1;
			//if /*untruncated*/ len == name.len() { return }
			self.account.get_unchecked_mut(end)
		};
		*last = 0;
	}
}

impl ArcDpsCache {
	pub fn lock_read() -> RwLockReadGuard<'static, Self> {
		ARCDPS_CACHE.read()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write() -> RwLockWriteGuard<'static, Self> {
		ARCDPS_CACHE.write()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub extern "C-unwind" fn request_account_name(_data: *const c_void) {
		addonapi_stub!(evtc::EV_REQUEST_ACCOUNT_NAME("{:?}", _data));

		#[cfg(debug_assertions)]
		if !_data.is_null() {
			debug!("evtc requests expect an empty payload");
		}

		// TODO: lock is pretty unnecessary here...
		let account = Self::lock_read().account_name();
		if let Some(account) = account {
			unsafe {
				NexusHost::addonapi_event_raise(NexusHost::EV_ACCOUNT_NAME.as_ptr(), account.as_ptr() as *const _ as *const c_void)
			}
		}
	}

	pub extern "C-unwind" fn request_target_changed(_data: *const c_void) {
		addonapi_stub!(evtc::EV_REPLAY_ARCDPS_TARGET_CHANGED("{:?}", _data));
		#[cfg(debug_assertions)]
		if !_data.is_null() {
			debug!("evtc requests expect an empty payload");
		}

		let arcdps = Self::lock_read();
		if let Some(target) = &arcdps.last_target {
			unsafe {
				NexusHost::addonapi_event_raise(NexusHost::EV_ARCDPS_TARGET_CHANGED.as_ptr(), target.as_nexus_ptr() as *const c_void)
			}
		}
	}

	pub extern "C-unwind" fn request_squad_join(_data: *const c_void) {
		addonapi_stub!(evtc::EV_REPLAY_ARCDPS_SQUAD_JOIN("{:?}", _data));
		#[cfg(debug_assertions)]
		if !_data.is_null() {
			debug!("evtc requests expect an empty payload");
		}

		let arcdps = Self::lock_read();
		for member in arcdps.squad_members.values() {
			unsafe {
				NexusHost::addonapi_event_raise(NexusHost::EV_ARCDPS_SQUAD_JOIN.as_ptr(), member.as_nexus_ptr() as *const c_void)
			}
		}
	}

	/// Ignore events from from our own update
	fn deny_reentry<F: FnOnce(&Self) -> bool>(f: F) -> bool {
		match ARCDPS_CACHE.try_read() {
			Ok(arcdps) =>
				f(&arcdps),
			_ => false,
		}
	}

	pub unsafe extern "C" fn evtc_target_changed(update: *const NexusAgentUpdate) {
		if Self::deny_reentry(|arcdps| arcdps.last_target() == nonnull_const(update)) {
			return
		}

		let target = ArcDpsAgentUpdate::from_ptr(&update);

		let mut arcdps = Self::lock_write();
		arcdps.last_target = Some(target.clone());
	}

	pub unsafe extern "C" fn evtc_account_name(name: *const c_char) {
		if Self::deny_reentry(|arcdps| arcdps.account_name() == nonnull_const(name)) {
			return
		}

		let name = match cstr_opt(&name) {
			Some(name) => name,
			None => {
				error!("received null account name");
				return
			},
		};

		let mut arcdps = Self::lock_write();
		arcdps.set_account_name(name);
	}

	pub unsafe extern "C" fn evtc_squad_join(update: *const NexusAgentUpdate) {
		if Self::deny_reentry(|arcdps| arcdps.squad_members.values().any(|m| m.as_nexus_ptr() == update)) {
			return
		}
		let member = ArcDpsAgentUpdate::from_ptr(&update);

		let mut arcdps = Self::lock_write();
		arcdps.squad_members.insert(member.id, member.clone());
	}

	pub unsafe extern "C" fn evtc_squad_leave(update: *const NexusAgentUpdate) {
		let member = ArcDpsAgentUpdate::from_ptr(&update);

		let mut arcdps = Self::lock_write();
		arcdps.squad_members.remove(&member.id);
	}

	pub fn init() {
		unsafe {
			NexusHost::addonapi_event_subscribe(NexusHost::EV_REPLAY_ARCDPS_SQUAD_JOIN.as_ptr(), Self::request_squad_join);
			NexusHost::addonapi_event_subscribe(NexusHost::EV_REPLAY_ARCDPS_TARGET_CHANGED.as_ptr(), Self::request_target_changed);
			NexusHost::addonapi_event_subscribe(NexusHost::EV_REQUEST_ACCOUNT_NAME.as_ptr(), Self::request_account_name);
			NexusHost::addonapi_event_subscribe(NexusHost::EV_ARCDPS_SQUAD_JOIN.as_ptr(), transmute(Self::evtc_squad_join as unsafe extern "C" fn(_)));
			NexusHost::addonapi_event_subscribe(NexusHost::EV_ARCDPS_SQUAD_LEAVE.as_ptr(), transmute(Self::evtc_squad_leave as unsafe extern "C" fn(_)));
			NexusHost::addonapi_event_subscribe(NexusHost::EV_ARCDPS_TARGET_CHANGED.as_ptr(), transmute(Self::evtc_target_changed as unsafe extern "C" fn(_)));
			NexusHost::addonapi_event_subscribe(NexusHost::EV_ACCOUNT_NAME.as_ptr(), transmute(Self::evtc_account_name as unsafe extern "C" fn(_)));
		}
	}
}

impl NexusHost {
	pub fn evtc(event: Option<&Event>, src: Option<&Agent>, dst: Option<&Agent>, _skill_name: PCSTR, id: u64, revision: u64, is_local: bool) {
		match event {
			None => {
				let src = match src {
					Some(src) => src,
					None => {
						warn!("unrecognized combat event id={id}, rev={revision}, dst={dst:?}");
						return
					},
				};
				let mut payload = ArcDpsAgentUpdate {
					id: src.id,
					added: src.prof,
					target: src.elite,
					team: src.team,
					.. Default::default()
				};
				let targeted = payload.target == 1;
				let key = match () {
					_ if targeted =>
						Self::EV_ARCDPS_TARGET_CHANGED,
					_ if payload.added != 0 =>
						Self::EV_ARCDPS_SQUAD_JOIN,
					_ =>
						Self::EV_ARCDPS_SQUAD_LEAVE,
					// TODO: more events probably...
				};
				if let Some(character) = unsafe { cstr_opt(&src.name_ptr()) } {
					cstr_write(&mut payload.character, character);
				}
				match dst {
					Some(dst) if payload.added != 0 => {
						if targeted {
							warn!("arcdps targeted event added={}, target={}? dst={dst:?}", payload.added, payload.target);
						}
						if let Some(account) = unsafe { cstr_opt(&dst.name_ptr()) } {
							cstr_write(&mut payload.account, account);
							if dst.is_self != 0 {
								Self::event_broadcast(Self::EV_ACCOUNT_NAME, account.as_ptr() as *const _);
							}
						}
						payload.instance_id = dst.id;
						payload.prof = dst.prof;
						payload.elite = dst.elite;
						payload.is_self = dst.is_self;
						payload.subgroup = dst.team;
					},
					None if payload.added != 0 => {
						warn!("arcdps add event included no dst agent; id={id}, rev={revision}, src={src:?}");
					},
					_ => (),
				}
				let payload = unsafe { payload.as_nexus() };
				Self::event_broadcast(key, payload as *const NexusAgentUpdate as *const _)
			},
			event @ Some(..) => {
				// TODO: pay attention to revision and/or id?
				let key = match is_local {
					true => Self::EV_ARCDPS_COMBATEVENT_LOCAL_RAW,
					false => Self::EV_ARCDPS_COMBATEVENT_SQUAD_RAW,
				};
				let payload = ArcDpsCombatData {
					event,
					src,
					dst,
					id,
					revision,
				};
				let payload = payload.as_nexus();
				Self::event_broadcast(key, payload as *const NexusCombatData as *const _)
			},
		}
	}
}

/// Representation of [NexusCombatData]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ArcDpsCombatData<'a> {
	pub event: Option<&'a Event>,
	pub src: Option<&'a Agent>,
	pub dst: Option<&'a Agent>,
	pub id: u64,
	pub revision: u64,
}

impl<'a> ArcDpsCombatData<'a> {
	#[inline]
	pub fn as_nexus(&self) -> &NexusCombatData {
		unsafe {
			debug_assert_eq!(size_of::<Self>(), size_of::<NexusCombatData>());
			transmute(self)
		}
	}
}

impl<'a> AsRef<NexusCombatData> for ArcDpsCombatData<'a> {
	fn as_ref(&self) -> &NexusCombatData {
		self.as_nexus()
	}
}

/// Representation of [NexusAgentUpdate]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ArcDpsAgentUpdate {
	pub account: [c_char; 64],
	pub character: [c_char; 64],
	pub id: usize,
	pub instance_id: usize,
	pub added: u32,
	pub target: u32,
	pub is_self: u32,
	pub prof: u32,
	pub elite: u32,
	pub team: u16,
	pub subgroup: u16,
}

impl ArcDpsAgentUpdate {
	pub unsafe fn from_ptr(nexus: &*const NexusAgentUpdate) -> &Self {
		let ptr: *const NexusAgentUpdate = *nexus;
		transmute(ptr)
	}

	pub unsafe fn as_nexus(&self) -> &NexusAgentUpdate {
		debug_assert_eq!(size_of::<Self>(), size_of::<NexusAgentUpdate>());
		transmute(self)
	}

	pub fn as_nexus_ptr(&self) -> *const NexusAgentUpdate {
		debug_assert_eq!(size_of::<Self>(), size_of::<NexusAgentUpdate>());
		unsafe {
			transmute(self)
		}
	}
}

impl Default for ArcDpsAgentUpdate {
	#[inline]
	fn default() -> Self {
		unsafe {
			MaybeUninit::zeroed().assume_init()
		}
	}
}
