use core::{fmt, mem::transmute, num::NonZeroU64};
use std::{borrow::{Borrow, Cow}, mem::MaybeUninit};
use arcffi::{c_bool32, c_char, cstr_write, CStr, CStrBox, CStrPtr, CStrRef};

#[cfg(feature = "evtc")]
pub use crate::arcdps::imp_evtc::{
	self as imp_evtc,
	Event as ImpCombatEventData,
};
#[cfg(feature = "evtc")]
pub use crate::arcdps::imp_realtime::{
	self as imp,
	Agent as ImpCombatAgent,
	AgentOwned as ImpCombatAgentOwned,
};

#[cfg(feature = "nexus")]
pub use ::nexus::event::arc::{
	AgentUpdate as ImpCombatEventAgent,
	CombatData as ImpCombatArgs,
};

#[doc(alias = "cbtevent")]
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatEventData {
	pub time: u64,
	pub src_agent: u64,
	pub dst_agent: u64,
	pub value: i32,
	pub buff_dmt: i32,
	pub overstack_value: u32,
	pub skillid: u32,
	pub src_instid: u16,
	pub dst_instid: u16,
	pub src_master_instid: u16,
	pub dst_master_instid: u16,
	pub iff: u8,
	pub buff: u8,
	pub result: u8,
	pub is_activation: u8,
	pub is_buffremove: u8,
	pub is_ninety: u8,
	pub is_fifty: u8,
	pub is_moving: u8,
	pub is_statechange: u8,
	pub is_flanking: u8,
	pub is_shields: u8,
	pub is_offcycle: u8,
	pub pad61: u8,
	pub pad62: u8,
	pub pad63: u8,
	pub pad64: u8,
}

impl CombatEventData {
	#[cfg(feature = "evtc")]
	pub const fn to_imp(self) -> ImpCombatEventData {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn as_imp(&self) -> &ImpCombatEventData {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn from_imp(ev: ImpCombatEventData) -> Self {
		unsafe {
			transmute(ev)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn from_imp_ref(ev: &ImpCombatEventData) -> &Self {
		unsafe {
			transmute(ev)
		}
	}

	#[cfg(feature = "evtc")]
	pub fn from_imp_mut(ev: &mut ImpCombatEventData) -> &mut Self {
		unsafe {
			transmute(ev)
		}
	}
}

#[cfg(feature = "evtc")]
impl AsRef<ImpCombatEventData> for CombatEventData {
	fn as_ref(&self) -> &ImpCombatEventData {
		self.as_imp()
	}
}

#[cfg(feature = "evtc")]
impl AsRef<CombatEventData> for ImpCombatEventData {
	fn as_ref(&self) -> &CombatEventData {
		CombatEventData::from_imp_ref(self)
	}
}

#[doc(alias = "ag")]
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatAgentRef<'n> {
	pub name: Option<CStrPtr<'n>>,
	pub id: usize,
	pub prof: u32,
	pub elite: u32,
	pub is_self: c_bool32,
	pub team: u16,
}

impl<'n> CombatAgentRef<'n> {
	#[inline]
	pub fn cloned(self) -> CombatAgent {
		CombatAgent::with_agent(self)
	}

	#[inline]
	pub const fn agent_ref(&self) -> &CombatAgent {
		CombatAgent::new_ref(self)
	}

	pub const fn unnamed(self) -> CombatAgentRef<'static> {
		CombatAgentRef {
			name: None,
			.. self
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn as_imp(&self) -> &ImpCombatAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const unsafe fn to_imp_unchecked(self) -> ImpCombatAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn from_imp_ref(agent: &'n ImpCombatAgent) -> &'n Self {
		unsafe {
			transmute(agent)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn from_imp(agent: &'n ImpCombatAgent) -> Self {
		*Self::from_imp_ref(agent)
	}

	#[cfg(feature = "evtc")]
	pub const unsafe fn from_imp_unchecked(agent: ImpCombatAgent) -> Self {
		unsafe {
			transmute(agent)
		}
	}
}

impl<'n> From<&'n CombatAgent> for CombatAgentRef<'n> {
	fn from(agent: &'n CombatAgent) -> Self {
		*agent.agent_ref()
	}
}

impl AsRef<CombatAgent> for CombatAgentRef<'_> {
	fn as_ref(&self) -> &CombatAgent {
		self.agent_ref()
	}
}

impl Borrow<CombatAgent> for CombatAgentRef<'_> {
	fn borrow(&self) -> &CombatAgent {
		CombatAgent::new_ref(self)
	}
}

#[cfg(feature = "evtc")]
impl AsRef<ImpCombatAgent> for CombatAgentRef<'_> {
	fn as_ref(&self) -> &ImpCombatAgent {
		self.as_imp()
	}
}

impl fmt::Debug for CombatAgentRef<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
		fmt::Debug::fmt(self.agent_ref(), f)
	}
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatAgent {
	pub name: Option<CStrBox>,
	pub id: usize,
	pub prof: u32,
	pub elite: u32,
	pub is_self: c_bool32,
	pub team: u16,
}

impl CombatAgent {
	pub fn with_agent(agent: CombatAgentRef<'_>) -> Self {
		match agent {
			mut agent => {
				agent.name = agent.name.map(|n| CStrBox::new(n).into_ptr());
				unsafe {
					Self::new_unchecked(agent)
				}
			},
			#[cfg(todo)]
			agent => Self {
				name: agent.name.map(|n| CStrBox::new(n)),
				.. Self::without_name(agent)
			},
			#[cfg(todo)]
			agent => Self::new_ref(&agent).clone(),
		}
	}

	pub const fn without_name(agent: CombatAgentRef<'_>) -> Self {
		Self {
			name: None,
			id: agent.id,
			prof: agent.prof,
			elite: agent.elite,
			is_self: agent.is_self,
			team: agent.team,
		}
	}

	#[inline]
	pub const unsafe fn new_unchecked(agent: CombatAgentRef<'_>) -> Self {
		transmute(agent)
	}

	#[inline]
	pub const fn new_ref<'a>(agent: &'a CombatAgentRef<'_>) -> &'a Self {
		unsafe {
			transmute(agent)
		}
	}

	#[inline]
	pub fn agent_ref<'n>(&'n self) -> &'n CombatAgentRef<'n> {
		unsafe {
			transmute(self)
		}
	}

	pub fn agent_event<'a>(&'a self, dst: Option<&'a Self>) -> Result<CombatEventAgent<'a>, CombatEventTarget<'a>> {
		match c_bool32::with(self.elite) {
			c_bool32::TRUE => Err({
				#[cfg(feature = "log")]
				if let Some(dst) = dst {
					log::debug!("combat target event has unexpected dst {dst:?}");
				}
				CombatEventTarget {
					src: Cow::Borrowed(self),
				}
			}),
			_ => Ok(CombatEventAgent {
				src: Cow::Borrowed(self),
				dst: match c_bool32::with(self.prof).get() {
					true => Some({
						// agent added event
						#[cfg(feature = "log")]
						if dst.is_none() {
							log::debug!("combat agent add dst missing? {self:?}");
						}
						dst.map(Cow::Borrowed).unwrap_or_default()
					}),
					false => match dst {
						// agent removed event
						Some(dst) if *dst != Self::default() =>
							Some(Cow::Borrowed(dst)),
						_ => None,
					},
				},
			}),
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn as_imp(&self) -> &ImpCombatAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn leak_into_imp(self) -> ImpCombatAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "evtc")]
	pub const fn from_imp_ref(agent: &ImpCombatAgent) -> &Self {
		unsafe {
			transmute(agent)
		}
	}

	#[cfg(feature = "evtc")]
	pub fn with_imp(agent: &ImpCombatAgent) -> Self {
		Self::with_agent(CombatAgentRef::from_imp(agent))
	}

	#[cfg(feature = "evtc")]
	pub const unsafe fn from_imp_unchecked(agent: ImpCombatAgent) -> Self {
		unsafe {
			transmute(agent)
		}
	}
}

#[cfg(feature = "evtc")]
impl AsRef<ImpCombatAgent> for CombatAgent {
	fn as_ref(&self) -> &ImpCombatAgent {
		self.as_imp()
	}
}

#[cfg(feature = "evtc")]
impl AsRef<CombatAgent> for ImpCombatAgent {
	fn as_ref(&self) -> &CombatAgent {
		CombatAgent::from_imp_ref(self)
	}
}

impl<'n> From<CombatAgentRef<'n>> for CombatAgent {
	fn from(agent: CombatAgentRef<'n>) -> Self {
		agent.cloned()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatArgs<'c> {
	pub ev: Option<Cow<'c, CombatEventData>>,
	pub src: Option<Cow<'c, CombatAgent>>,
	pub dst: Option<Cow<'c, CombatAgent>>,
	pub skill_name: Cow<'c, CStrRef>,
	pub id: Option<NonZeroU64>,
	pub revision: u64,
}

impl<'c> CombatArgs<'c> {
	pub fn skill_name(&self) -> Option<&CStrRef> {
		match self.skill_name.is_empty() {
			true => None,
			false => Some(&self.skill_name),
		}
	}

	fn combat_event(&self) -> Result<&Cow<'c, CombatEventData>, Option<&Cow<'c, CombatAgent>>> {
		match &self.ev {
			Some(ev) => Ok(ev),
			None => Err(self.src.as_ref()),
		}
	}

	pub fn event(&self) -> Option<CombatEvent> {
		let dst = self.dst.as_ref().map(|a| &**a);
		Some(match self.combat_event() {
			Ok(ev) => CombatEvent::Skill(CombatEventSkill {
				ev: Cow::Borrowed(ev),
				src: self.src.as_ref().map(|a| Cow::Borrowed(&**a)),
				dst: dst.map(Cow::Borrowed),
				skill_name: Cow::Borrowed(&self.skill_name),
			}),
			Err(Some(src)) => {
				let agent_ev = src.agent_event(dst);
				let src = Cow::Borrowed(&**src);
				let dst = dst.map(Cow::Borrowed);
				match agent_ev {
					Err(_target) => CombatEvent::Target(CombatEventTarget {
						src,
					}),
					Ok(agent) => CombatEvent::Agent(CombatEventAgent {
						src,
						dst: match agent.dst {
							None => None,
							Some(d) => Some(dst.unwrap_or(d)),
						},
					}),
				}
			},
			Err(None) => return None,
		})
	}

	pub fn event_cloned(&self) -> Option<CombatEvent<'c>> {
		Some(match self.combat_event() {
			Ok(ev) => CombatEvent::Skill(CombatEventSkill {
				ev: ev.clone(),
				src: self.src.clone(),
				dst: self.dst.clone(),
				skill_name: self.skill_name.clone(),
			}),
			Err(Some(src)) => {
				let agent_ev = src.agent_event(self.dst.as_ref().map(|d| &**d));
				let src = src.clone();
				let dst = self.dst.clone();
				match agent_ev {
					Err(_target) => CombatEvent::Target(CombatEventTarget {
						src,
					}),
					Ok(agent) => CombatEvent::Agent(CombatEventAgent {
						src,
						dst: match agent.dst {
							None => None,
							Some(d) => Some(dst.unwrap_or_else(|| Cow::Owned(d.into_owned()))),
						},
					}),
				}
			},
			Err(None) => return None,
		})
	}

	pub fn into_owned(self) -> CombatArgs<'static> {
		CombatArgs {
			ev: self.ev.map(Cow::into_owned).map(Cow::Owned),
			src: self.src.map(Cow::into_owned).map(Cow::Owned),
			dst: self.dst.map(Cow::into_owned).map(Cow::Owned),
			skill_name: Cow::Owned(self.skill_name.into_owned()),
			id: self.id,
			revision: self.revision,
		}
	}

	#[cfg(feature = "nexus")]
	pub fn with_imp(args: &'c ImpCombatArgs) -> Self {
		let (ev, src, dst, id, revision) = args.as_tuple();

		Self {
			#[cfg(feature = "evtc")]
			ev: ev.map(CombatEventData::from_imp_ref)
				.map(Cow::Borrowed),
			#[cfg(feature = "evtc")]
			src: src.map(CombatAgent::from_imp_ref)
				.map(Cow::Borrowed),
			#[cfg(feature = "evtc")]
			dst: dst.map(CombatAgent::from_imp_ref)
				.map(Cow::Borrowed),
			#[cfg(not(feature = "evtc"))]
			ev: ev.map(|a| unsafe { transmute(a) })
				.map(Cow::Borrowed),
			#[cfg(not(feature = "evtc"))]
			src: src.map(|a| unsafe { transmute(a) })
				.map(Cow::Borrowed),
			#[cfg(not(feature = "evtc"))]
			dst: dst.map(|a| unsafe { transmute(a) })
				.map(Cow::Borrowed),
			skill_name: Cow::Borrowed(CStrRef::EMPTY),
			id: NonZeroU64::new(id),
			revision,
		}
	}

	#[cfg(feature = "nexus")]
	pub unsafe fn to_imp_unchecked(&self) -> ImpCombatArgs {
		NexusCombatArgs::with_args(self).to_imp_unchecked()
	}

	#[cfg(feature = "nexus")]
	pub fn borrow_imp<R, F>(&self, f: F) -> R where
		F: FnOnce(&ImpCombatArgs) -> R,
	{
		let args = unsafe { self.to_imp_unchecked() };
		f(&args)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CombatEvent<'c> {
	Skill(CombatEventSkill<'c>),
	Target(CombatEventTarget<'c>),
	Agent(CombatEventAgent<'c>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatEventSkill<'c> {
	pub ev: Cow<'c, CombatEventData>,
	pub src: Option<Cow<'c, CombatAgent>>,
	pub dst: Option<Cow<'c, CombatAgent>>,
	pub skill_name: Cow<'c, CStrRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatEventAgent<'c> {
	pub src: Cow<'c, CombatAgent>,
	pub dst: Option<Cow<'c, CombatAgent>>,
}

impl<'c> CombatEventAgent<'c> {
	pub fn is_added(&self) -> bool {
		self.dst.is_some() && self.src.prof != 0
	}

	pub fn is_self(&self) -> c_bool32 {
		self.dst.as_ref()
			.map(|d| d.is_self)
			.unwrap_or(c_bool32::FALSE)
	}

	pub fn agent(&self) -> CombatAgentRef {
		let mut agent = CombatAgentRef {
			name: Default::default(),
			prof: Default::default(),
			elite: Default::default(),
			id: self.src.id,
			team: self.src.team,
			is_self: self.src.is_self,
		};
		if let Some(dst) = &self.dst {
			agent.is_self = dst.is_self;
			agent.name = dst.name.as_ref().map(|name| name.as_c_ptr());
			agent.prof = dst.prof;
			agent.elite = dst.elite;
		}
		agent
	}

	pub fn id(&self) -> usize {
		self.src.id
	}

	pub fn character_name(&self) -> Option<CStrPtr> {
		self.src.name.as_ref()
			.map(|n| n.as_c_ptr())
	}

	pub fn account_names(&self) -> Option<CStrPtr> {
		self.dst.as_ref()
			.and_then(|d| d.name.as_ref())
			.map(|n| n.as_c_ptr())
	}

	pub fn instance_id_on_map(&self) -> Option<usize> {
		self.dst.as_ref()
			.map(|d| d.id)
	}

	pub fn profession(&self) -> Option<u32> {
		self.dst.as_ref()
			.map(|d| d.prof)
	}

	pub fn elite_spec(&self) -> Option<u32> {
		self.dst.as_ref()
			.map(|d| d.elite)
	}

	pub fn team_id(&self) -> u16 {
		self.src.team
	}

	pub fn subgroup(&self) -> Option<u16> {
		self.dst.as_ref()
			.map(|d| d.team)
	}

	pub fn to_nexus(&self) -> NexusCombatEventAgent {
		let mut payload = NexusCombatEventAgent {
			id: self.id(),
			added: c_bool32::new(self.is_added()),
			target: c_bool32::FALSE,
			team: self.team_id(),
			instance_id: self.instance_id_on_map().unwrap_or_default(),
			prof: self.profession().unwrap_or_default(),
			elite: self.elite_spec().unwrap_or_default(),
			is_self: self.is_self(),
			subgroup: self.subgroup().unwrap_or_default(),
			.. Default::default()
		};
		if let Some(name) = self.character_name() {
			cstr_write(&mut payload.character, name.to_c_str());
		}
		if let Some(name) = self.account_names() {
			cstr_write(&mut payload.account, name.to_c_str());
		}
		payload
	}

	#[cfg(todo)]
	pub fn from_nexus(event: &NexusCombatEventAgent) -> Option<Self> {
	}

	#[cfg(feature = "nexus")]
	pub fn to_imp(&self) -> ImpCombatEventAgent {
		unsafe {
			self.to_nexus().to_imp_unchecked()
		}
	}
}

/// Representation of [ImpCombatEventAgent]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NexusCombatEventAgent {
	pub account: [c_char; 64],
	pub character: [c_char; 64],
	pub id: usize,
	pub instance_id: usize,
	pub added: c_bool32,
	pub target: c_bool32,
	pub is_self: c_bool32,
	pub prof: u32,
	pub elite: u32,
	pub team: u16,
	pub subgroup: u16,
}

impl NexusCombatEventAgent {
	pub fn account_names(&self) -> Option<&CStr> {
		let term = self.account.iter()
			.position(|&v| v == 0)?;
		Some(unsafe {
			let bytes = self.account.get_unchecked(..=term);
			CStr::from_bytes_with_nul_unchecked(transmute(bytes))
		})
	}

	pub fn character_name(&self) -> Option<&CStr> {
		let term = self.character.iter()
			.position(|&v| v == 0)?;
		Some(unsafe {
			let bytes = self.account.get_unchecked(..=term);
			CStr::from_bytes_with_nul_unchecked(transmute(bytes))
		})
	}

	#[cfg(feature = "nexus")]
	pub const fn from_imp_ref(event: &ImpCombatEventAgent) -> &Self {
		unsafe {
			transmute(event)
		}
	}

	#[cfg(feature = "nexus")]
	pub const fn from_imp(event: ImpCombatEventAgent) -> Self {
		unsafe {
			transmute(event)
		}
	}

	#[cfg(feature = "nexus")]
	pub const unsafe fn to_imp_unchecked(self) -> ImpCombatEventAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "nexus")]
	pub const unsafe fn as_imp_unchecked(&self) -> &ImpCombatEventAgent {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "nexus")]
	pub fn to_imp(self) -> Option<ImpCombatEventAgent> {
		let _ = self.character_name()?;
		let _ = self.account_names()?;
		Some(unsafe {
			self.to_imp_unchecked()
		})
	}

	#[cfg(feature = "nexus")]
	pub fn as_imp(&self) -> Option<&ImpCombatEventAgent> {
		let _ = self.character_name()?;
		let _ = self.account_names()?;
		Some(unsafe {
			self.as_imp_unchecked()
		})
	}
}

impl Default for NexusCombatEventAgent {
	#[inline]
	fn default() -> Self {
		unsafe {
			MaybeUninit::zeroed().assume_init()
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CombatEventTarget<'c> {
	pub src: Cow<'c, CombatAgent>,
}

impl<'c> CombatEventTarget<'c> {
	pub fn target_id(&self) -> usize {
		self.src.id
	}

	/// TODO: unclear if character_name, is_self, team, etc are ever valid here?
	pub fn to_nexus(&self) -> NexusCombatEventAgent {
		let payload = NexusCombatEventAgent {
			id: self.target_id(),
			target: c_bool32::TRUE,
			#[cfg(todo)]
			is_self: self.src.is_self,
			#[cfg(todo)]
			team: self.src.team,
			.. Default::default()
		};
		#[cfg(todo)]
		if let Some(name) = &self.src.name {
			cstr_write(&mut payload.character, name.to_c_str());
		}
		payload
	}

	#[cfg(todo)]
	pub fn from_nexus(event: &NexusCombatEventAgent) -> Option<Self> {
	}

	#[cfg(feature = "nexus")]
	pub fn to_imp(&self) -> ImpCombatEventAgent {
		unsafe {
			self.to_nexus().to_imp_unchecked()
		}
	}
}

#[cfg(todo)]
impl<'c> Deref for CombatEventTarget<'c> {
	type Target = CombatAgent;

	fn deref(&self) -> &CombatAgent {
		&self.src
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NexusCombatArgs<'c> {
	pub ev: Option<&'c CombatEventData>,
	pub src: Option<&'c CombatAgent>,
	pub dst: Option<&'c CombatAgent>,
	pub id: Option<NonZeroU64>,
	pub revision: u64,
}

impl<'c> NexusCombatArgs<'c> {
	pub fn with_args(args: &'c CombatArgs) -> Self {
		Self {
			ev: args.ev.as_ref().map(|ev| &**ev),
			src: args.src.as_ref().map(|src| &**src),
			dst: args.dst.as_ref().map(|dst| &**dst),
			id: args.id,
			revision: args.revision,
		}
	}

	#[cfg(feature = "nexus")]
	pub const unsafe fn to_imp_unchecked(self) -> ImpCombatArgs {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "nexus")]
	pub const fn as_imp(&self) -> &ImpCombatArgs {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "nexus")]
	pub const unsafe fn from_imp_unchecked(ev: ImpCombatArgs) -> Self {
		unsafe {
			transmute(ev)
		}
	}

	#[cfg(feature = "nexus")]
	pub const fn from_imp(ev: &'c ImpCombatArgs) -> Self {
		*Self::from_imp_ref(ev)
	}

	#[cfg(feature = "nexus")]
	pub const fn from_imp_ref(ev: &'c ImpCombatArgs) -> &'c Self {
		unsafe {
			transmute(ev)
		}
	}

	#[cfg(feature = "nexus")]
	pub fn from_imp_mut(ev: &mut ImpCombatArgs) -> &mut Self {
		unsafe {
			transmute(ev)
		}
	}
}

#[cfg(feature = "nexus")]
impl AsRef<ImpCombatArgs> for NexusCombatArgs<'_> {
	fn as_ref(&self) -> &ImpCombatArgs {
		self.as_imp()
	}
}
