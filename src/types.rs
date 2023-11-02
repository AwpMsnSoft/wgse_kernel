#![allow(unused_macros)]

use crate::errors::{inconsistent_type_err, InconsistentTypeError};
use anyhow::{anyhow, Result};
use binrw::{
    binrw, io as binio, BinRead, BinResult, BinWrite, Endian as BinEndian, Error as BinError,
    NullString,
};
use derive_deref::{Deref, DerefMut};
use enum_dispatch::enum_dispatch;
use function_name::named;
use leb128::{
    read::{self as sleb128, signed as sleb128_read},
    write::signed as sleb128_write,
};
use std::{
    any::type_name,
    io::{self, Cursor},
    marker::PhantomData,
    num::NonZeroU32,
};

pub mod wrapper {
    use super::*;

    #[binrw]
    #[derive(Copy, Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Opcode(u8);

    #[binrw]
    #[derive(Copy, Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Register(u64);

    #[binrw]
    #[brw(little)]
    #[repr(C, align(1))]
    #[derive(Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq)]
    pub struct BinVec<T>
    where
        T: 'static + for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        #[bw(calc = buff.len() as u64)]
        cnt: u64,
        #[br(count=  cnt)]
        buff: Vec<T>,
    }

    impl<T> From<BinVec<T>> for Vec<T>
    where
        T: 'static + for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        fn from(value: BinVec<T>) -> Self {
            value.buff
        }
    }

    impl<T> From<Vec<T>> for BinVec<T>
    where
        T: 'static + for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        fn from(value: Vec<T>) -> Self {
            Self { buff: value }
        }
    }
}

pub mod common {
    use super::wrapper::*;
    use super::*;

    #[derive(Copy, Clone, Debug, Default, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Integer(i64);

    impl BinWrite for Integer {
        type Args<'a> = ();

        fn write_options<W: binio::Write + binio::Seek>(
            &self,
            writer: &mut W,
            endian: BinEndian,
            _args: Self::Args<'_>,
        ) -> BinResult<()> {
            match endian {
                BinEndian::Little => {
                    sleb128_write(writer, **self)?;
                    Ok(())
                }
                BinEndian::Big => Err(BinError::Io(binio::Error::new(
                    binio::ErrorKind::InvalidInput,
                    "leb_128 can ONLY be little endian",
                ))),
            }
        }
    }

    impl BinRead for Integer {
        type Args<'a> = ();

        fn read_options<R: binio::Read + binio::Seek>(
            reader: &mut R,
            endian: BinEndian,
            _args: Self::Args<'_>,
        ) -> BinResult<Self> {
            match endian {
                BinEndian::Little => sleb128_read(reader).map(Self).map_err(|err| match err {
                    sleb128::Error::IoError(err) => BinError::Io(binio::Error::new(
                        binio::ErrorKind::InvalidData,
                        format!("{:?}", err),
                    )),
                    sleb128::Error::Overflow => BinError::Io(binio::Error::new(
                        binio::ErrorKind::InvalidData,
                        "the number being read is larger than can be represented",
                    )),
                }),
                BinEndian::Big => Err(BinError::Io(binio::Error::new(
                    binio::ErrorKind::InvalidInput,
                    "leb_128 can ONLY be little endian",
                ))),
            }
        }
    }

    #[binrw]
    #[brw(little)]
    #[derive(Copy, Clone, Debug, Default, Deref, DerefMut, PartialEq, PartialOrd)]
    pub struct Real(f64);

    #[derive(Copy, Clone, Debug, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Address(NonZeroU32);

    impl BinWrite for Address {
        type Args<'a> = ();

        fn write_options<W: binio::Write + binio::Seek>(
            &self,
            writer: &mut W,
            endian: BinEndian,
            _args: Self::Args<'_>,
        ) -> BinResult<()> {
            match endian {
                BinEndian::Little => {
                    sleb128_write(writer, u32::from(**self) as i64)?;
                    Ok(())
                }
                BinEndian::Big => Err(BinError::Io(binio::Error::new(
                    binio::ErrorKind::InvalidInput,
                    "leb_128 can ONLY be little endian",
                ))),
            }
        }
    }

    impl BinRead for Address {
        type Args<'a> = ();

        fn read_options<R: binio::Read + binio::Seek>(
            reader: &mut R,
            endian: BinEndian,
            _args: Self::Args<'_>,
        ) -> BinResult<Self> {
            const U32_MAX: i64 = u32::MAX as i64;
            match endian {
                BinEndian::Little => sleb128_read(reader)
                    .and_then(|val| match val {
                        addr @ 1..=U32_MAX => Ok(Self(
                            // SAFETY: always safe
                            unsafe { NonZeroU32::new_unchecked(addr as u32) },
                        )),
                        addr => Err(sleb128::Error::IoError(io::Error::new(
                            io::ErrorKind::Other,
                            anyhow!(
                                "address range exceeded. except [1..=0xFFFFFFFF], found [0x{:X}]",
                                addr
                            ),
                        ))),
                    })
                    .map_err(|err| match err {
                        sleb128::Error::IoError(err) => BinError::Io(binio::Error::new(
                            binio::ErrorKind::InvalidData,
                            format!("{:?}", err),
                        )),
                        sleb128::Error::Overflow => BinError::Io(binio::Error::new(
                            binio::ErrorKind::InvalidData,
                            "the number being read is larger than can be represented",
                        )),
                    }),
                BinEndian::Big => Err(BinError::Io(binio::Error::new(
                    binio::ErrorKind::InvalidInput,
                    "leb_128 can ONLY be little endian",
                ))),
            }
        }
    }

    #[cfg(debug_assertions)]
    #[binrw]
    #[brw(little)]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Handle<T>
    where
        T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        pub address: u32,
        pub type_info: NullString,
        #[brw(ignore)]
        phantom: PhantomData<T>,
    }

    #[cfg(not(debug_assertions))]
    #[binrw]
    #[brw(little)]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Handle<T>
    where
        T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        pub address: u32,
        #[brw(ignore)]
        phantom: PhantomData<T>,
    }

    impl<T> Handle<T>
    where
        T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
    {
        #[cfg(debug_assertions)]
        fn new(addr: u32) -> Self {
            Self {
                address: addr,
                type_info: type_name::<Handle<T>>().into(),
                ..Default::default()
            }
        }

        #[cfg(not(debug_assertions))]
        fn new(addr: u32) -> Self {
            Self {
                address: addr,
                ..Default::default()
            }
        }
    }

    #[binrw]
    #[brw(little)]
    #[enum_dispatch(TryDerefTo)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Value {
        Integer(Integer),
        Real(Real),
        Address(Address),
    }

    #[enum_dispatch]
    pub trait TryDerefTo {
        #[named]
        fn try_to_integer(&self) -> Result<i64> {
            Err(inconsistent_type_err!(
                type_name::<Integer>(),
                type_name::<Self>()
            ))
        }

        #[named]
        fn try_to_real(&self) -> Result<f64> {
            Err(inconsistent_type_err!(
                type_name::<Real>(),
                type_name::<Self>()
            ))
        }

        #[named]
        fn try_to_handle<T>(&self) -> Result<Handle<T>>
        where
            T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
        {
            Err(inconsistent_type_err!(
                type_name::<Handle::<T>>(),
                type_name::<Self>()
            ))
        }
    }

    impl TryDerefTo for Integer {
        fn try_to_integer(&self) -> Result<i64> {
            Ok(**self)
        }
    }

    impl TryDerefTo for Real {
        fn try_to_real(&self) -> Result<f64> {
            Ok(**self)
        }
    }

    impl TryDerefTo for Address {
        fn try_to_handle<T>(&self) -> Result<Handle<T>>
        where
            T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
        {
            Ok(Handle::<T>::new(u32::from(**self)))
        }
    }

    #[binrw]
    #[brw(little)]
    #[repr(C, align(1))]
    #[derive(Clone, Debug, Default, Deref, DerefMut, Eq, PartialEq)]
    pub struct Argument(BinVec<u8>);

    impl Argument {
        pub fn get<T>(&mut self) -> Result<T>
        where
            T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
        {
            let mut cursor = Cursor::new(self.as_mut_slice());
            Ok(T::read_le(&mut cursor)?)
        }

        pub fn set<T>(&mut self, t: T) -> Result<()>
        where
            T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Default,
        {
            self.truncate(0);
            let mut cursor = Cursor::new(&mut ***self);
            t.write_le(&mut cursor)?;
            Ok(())
        }
    }

    #[binrw]
    #[brw(little)]
    #[repr(C, align(1))]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Instruction {
        pub opcode: Opcode,
        pub arguments: BinVec<Argument>,
    }
}
