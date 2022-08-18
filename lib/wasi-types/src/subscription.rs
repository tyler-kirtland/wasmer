use crate::*;
use std::convert::TryFrom;
use std::fmt;
use std::mem::{self, MaybeUninit};
use wasmer_derive::ValueType;
use wasmer_types::ValueType;
use wasmer_wasi_types_generated::wasi_snapshot0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueType)]
#[repr(C)]
pub struct __wasi_subscription_clock_t {
    pub clock_id: wasi_snapshot0::Clockid,
    pub timeout: __wasi_timestamp_t,
    pub precision: __wasi_timestamp_t,
    pub flags: wasi_snapshot0::Subclockflags,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueType)]
#[repr(C)]
pub struct __wasi_subscription_fs_readwrite_t {
    pub fd: __wasi_fd_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union __wasi_subscription_u {
    pub clock: __wasi_subscription_clock_t,
    pub fd_readwrite: __wasi_subscription_fs_readwrite_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __wasi_subscription_t {
    pub userdata: __wasi_userdata_t,
    pub type_: wasi_snapshot0::Eventtype,
    pub u: __wasi_subscription_u,
}

/// Safe Rust wrapper around `__wasi_subscription_t::type_` and `__wasi_subscription_t::u`
#[derive(Debug, Clone)]
pub enum EventType {
    Clock(__wasi_subscription_clock_t),
    Read(__wasi_subscription_fs_readwrite_t),
    Write(__wasi_subscription_fs_readwrite_t),
}

impl EventType {
    pub fn raw_tag(&self) -> wasi_snapshot0::Eventtype {
        match self {
            EventType::Clock(_) => wasi_snapshot0::Eventtype::Clock,
            EventType::Read(_) => wasi_snapshot0::Eventtype::FdRead,
            EventType::Write(_) => wasi_snapshot0::Eventtype::FdWrite,
        }
    }
}

/// Safe Rust wrapper around `__wasi_subscription_t`
#[derive(Debug, Clone)]
pub struct WasiSubscription {
    pub user_data: __wasi_userdata_t,
    pub event_type: EventType,
}

impl TryFrom<__wasi_subscription_t> for WasiSubscription {
    type Error = wasi_snapshot0::Errno;

    fn try_from(ws: __wasi_subscription_t) -> Result<Self, Self::Error> {
        Ok(Self {
            user_data: ws.userdata,
            event_type: match ws.type_ {
                wasi_snapshot0::Eventtype::Clock => EventType::Clock(unsafe { ws.u.clock }),
                wasi_snapshot0::Eventtype::FdRead => EventType::Read(unsafe { ws.u.fd_readwrite }),
                wasi_snapshot0::Eventtype::FdWrite => {
                    EventType::Write(unsafe { ws.u.fd_readwrite })
                }
            },
        })
    }
}

impl TryFrom<WasiSubscription> for __wasi_subscription_t {
    type Error = wasi_snapshot0::Errno;

    fn try_from(ws: WasiSubscription) -> Result<Self, Self::Error> {
        #[allow(unreachable_patterns)]
        let (type_, u) = match ws.event_type {
            EventType::Clock(c) => (
                wasi_snapshot0::Eventtype::Clock,
                __wasi_subscription_u { clock: c },
            ),
            EventType::Read(rw) => (
                wasi_snapshot0::Eventtype::FdRead,
                __wasi_subscription_u { fd_readwrite: rw },
            ),
            EventType::Write(rw) => (
                wasi_snapshot0::Eventtype::FdWrite,
                __wasi_subscription_u { fd_readwrite: rw },
            ),
            _ => return Err(wasi_snapshot0::Errno::Inval),
        };

        Ok(Self {
            userdata: ws.user_data,
            type_,
            u,
        })
    }
}

impl fmt::Debug for __wasi_subscription_t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("__wasi_subscription_t")
            .field("userdata", &self.userdata)
            .field("type", &eventtype_to_str(self.type_))
            .field(
                "u",
                match self.type_ {
                    wasi_snapshot0::Eventtype::Clock=> unsafe { &self.u.clock },
                    wasi_snapshot0::Eventtype::FdRead | wasi_snapshot0::Eventtype::FdWrite => unsafe {
                        &self.u.fd_readwrite
                    },
                },
            )
            .finish()
    }
}

unsafe impl ValueType for __wasi_subscription_t {
    fn zero_padding_bytes(&self, bytes: &mut [MaybeUninit<u8>]) {
        macro_rules! field {
            ($($f:tt)*) => {
                &self.$($f)* as *const _ as usize - self as *const _ as usize
            };
        }
        macro_rules! field_end {
            ($($f:tt)*) => {
                field!($($f)*) + mem::size_of_val(&self.$($f)*)
            };
        }
        macro_rules! zero {
            ($start:expr, $end:expr) => {
                for i in $start..$end {
                    bytes[i] = MaybeUninit::new(0);
                }
            };
        }
        self.userdata
            .zero_padding_bytes(&mut bytes[field!(userdata)..field_end!(userdata)]);
        zero!(field_end!(userdata), field!(type_));
        self.type_
            .zero_padding_bytes(&mut bytes[field!(type_)..field_end!(type_)]);
        zero!(field_end!(type_), field!(u));
        match self.type_ {
            wasi_snapshot0::Eventtype::FdRead | wasi_snapshot0::Eventtype::FdWrite => unsafe {
                self.u.fd_readwrite.zero_padding_bytes(
                    &mut bytes[field!(u.fd_readwrite)..field_end!(u.fd_readwrite)],
                );
                zero!(field_end!(u.fd_readwrite), field_end!(u));
            },
            wasi_snapshot0::Eventtype::Clock => unsafe {
                self.u
                    .clock
                    .zero_padding_bytes(&mut bytes[field!(u.clock)..field_end!(u.clock)]);
                zero!(field_end!(u.clock), field_end!(u));
            },
        }
        zero!(field_end!(u), mem::size_of_val(self));
    }
}

pub enum SubscriptionEnum {
    Clock(__wasi_subscription_clock_t),
    FdReadWrite(__wasi_subscription_fs_readwrite_t),
}

impl __wasi_subscription_t {
    pub fn tagged(&self) -> Option<SubscriptionEnum> {
        match self.type_ {
            wasi_snapshot0::Eventtype::Clock => {
                Some(SubscriptionEnum::Clock(unsafe { self.u.clock }))
            }
            wasi_snapshot0::Eventtype::FdRead | wasi_snapshot0::Eventtype::FdWrite => {
                Some(SubscriptionEnum::FdReadWrite(unsafe {
                    self.u.fd_readwrite
                }))
            }
        }
    }
}
