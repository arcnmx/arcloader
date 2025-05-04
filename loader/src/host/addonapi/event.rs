use nexus::event::{arc::{CombatData as NexusCombatData, AgentUpdate as NexusAgentUpdate}, RawEventConsumeUnknown};
#[cfg(feature = "arcdps")]
use ::{
	arcdps::{Event, Agent},
	windows_strings::PCSTR,
	std::mem,
};
use crate::{
	host::addonapi::NexusHost,
	util::ffi::{cstr_opt, cstr_write},
};
use std::{ffi::{c_char, c_void, CStr}, mem::MaybeUninit};

impl NexusHost {
	pub const EV_ADDON_LOADED: &'static CStr = cstr!("EV_ADDON_LOADED");
	pub const EV_WINDOW_RESIZED: &'static CStr = cstr!("EV_WINDOW_RESIZED");
	pub const EV_MUMBLE_IDENTITY_UPDATED: &'static CStr = cstr!("EV_MUMBLE_IDENTITY_UPDATED");

	pub const EV_ARCDPS_COMBATEVENT_LOCAL_RAW: &'static CStr = cstr!("EV_ARCDPS_COMBATEVENT_LOCAL_RAW");
	pub const EV_ARCDPS_COMBATEVENT_SQUAD_RAW: &'static CStr = cstr!("EV_ARCDPS_COMBATEVENT_SQUAD_RAW");
	pub const EV_ARCDPS_TARGET_CHANGED: &'static CStr = cstr!("EV_ARCDPS_TARGET_CHANGED");
	pub const EV_ARCDPS_SQUAD_JOIN: &'static CStr = cstr!("EV_ARCDPS_SQUAD_JOIN");
	pub const EV_ARCDPS_SQUAD_LEAVE: &'static CStr = cstr!("EV_ARCDPS_SQUAD_LEAVE");

	#[cfg(feature = "arcdps")]
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

	pub unsafe extern "C-unwind" fn addonapi_event_subscribe(identifier: *const c_char, consume_callback: RawEventConsumeUnknown) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(event::subscribe("{:?}, {:?}", id, consume_callback));

		let id = match id {
			Some(id) => id,
			None => {
				warn!("expected event id");
				return
			},
		};

		Self::cache_write_with(consume_callback as *const _, move |mut cache| {
			let handlers = cache.event_handlers.entry(id.to_owned())
				.or_default();
			handlers.insert(consume_callback);
		});
	}

	pub unsafe extern "C-unwind" fn addonapi_event_unsubscribe(identifier: *const c_char, consume_callback: RawEventConsumeUnknown) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(event::unsubscribe("{:?}, {:?}", id, consume_callback));

		let id = match id {
			Some(id) => id,
			None => {
				warn!("expected event id");
				return
			},
		};

		let removed = Self::cache_write_with(consume_callback as *const _, move |mut cache| {
			cache.event_handlers.get_mut(id)
				.map(|h| h.remove(&consume_callback))
				.unwrap_or(false)
		});
		if !removed {
			warn!("subscriber not found");
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise(identifier: *const c_char, event_data: *const c_void) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise("{:?}, {:?}", id, event_data) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_targeted(signature: i32, identifier: *const c_char, event_data: *const c_void) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_targeted("{:?}, {:?}, {:?}", signature, id, event_data) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_notification(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_notification("{:?}", id) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_notification_targeted(signature: i32, identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_notification_targeted("{:?}, {:?}", signature, id) => ())
	}
}

/// Representation of [NexusCombatData]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[cfg(feature = "arcdps")]
pub struct ArcDpsCombatData<'a> {
	pub event: Option<&'a Event>,
	pub src: Option<&'a Agent>,
	pub dst: Option<&'a Agent>,
	pub id: u64,
	pub revision: u64,
}

#[cfg(feature = "arcdps")]
impl<'a> ArcDpsCombatData<'a> {
	#[inline]
	pub fn as_nexus(&self) -> &NexusCombatData {
		unsafe {
			debug_assert_eq!(size_of::<Self>(), size_of::<NexusCombatData>());
			mem::transmute(self)
		}
	}
}

#[cfg(feature = "arcdps")]
impl<'a> AsRef<NexusCombatData> for ArcDpsCombatData<'a> {
	fn as_ref(&self) -> &NexusCombatData {
		self.as_nexus()
	}
}

/// Representation of [NexusAgentUpdate]
#[cfg(feature = "arcdps")]
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
	pub unsafe fn as_nexus(&self) -> &NexusAgentUpdate {
		debug_assert_eq!(size_of::<Self>(), size_of::<NexusAgentUpdate>());
		mem::transmute(self)
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
