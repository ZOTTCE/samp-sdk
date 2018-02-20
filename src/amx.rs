/*!

*/

use std::ptr::read;
use std::mem::{transmute, transmute_copy};
use std::ffi::CString;
use types;
use types::Cell;
use consts::*;
use data::amx_functions;

pub type AmxResult<T> = Result<T, AmxError>;

macro_rules! ret {
    ($res:ident, $ret:expr) => {
        {
            if $res == 0 {
                Ok($ret)
            } else {
                Err(AmxError::from($res))
            }
        }
    }
}

macro_rules! call {
    (
        $ex:expr
        =>
        $ret:expr
    ) => {
        {
            let result = $ex;
            ret!(result, $ret)
        }
    }
}

macro_rules! import {
    ($type:ident) => {
        unsafe {
            read(amx_functions.offset(Exports::$type as isize) as *const $crate::types::$type)
        }
    };
}

/// AMX struct that holds raw `types::AMX` pointer.
pub struct AMX {
    amx: *mut types::AMX,
}

impl AMX {
    /// Convert raw `types::AMX` pointer.
    pub fn new(amx: *mut types::AMX) -> AMX {
        AMX {
            amx,
        }
    }

    /// Register natives functions
    ///
    /// # Examples
    /// 
    /// ```
    /// fn amx_load(amx: AMX) -> Cell {
    ///     let natives = natives!{
    ///         "SomeFunction" => some_function,
    ///         "AnotherFunction" => another_function   
    ///     };
    /// 
    ///     amx.register(&natives).unwrap();
    ///     
    ///     AMX_ERR_NONE
    /// }
    /// ```
    pub fn register(&self, natives: &Vec<types::AMX_NATIVE_INFO>) -> AmxResult<()> {
        let len = natives.len();
        let ptr = natives.as_slice().as_ptr();

        let register = import!(Register);
        call!(register(self.amx, ptr, len as i32) => ())
    }

    /// Get an address of a reference value given to native.
    ///
    /// You **must** use `std::mem::forget` for this value because `get_address` return `Box<T>` which releases memory.
    ///
    /// # Examples
    ///
    /// ```
    /// // native: SomeNative(&int_value);
    /// fn some_native(amx: AMX, args: *mut Cell) -> Cell {
    ///     let ptr = std::ptr::read(args.offset(1));
    ///     let int_value: Box<i32> = amx.get_address(ptr).unwrap();
    ///     *int_value = 10;
    ///     
    ///     std::mem::forget(int_value);
    /// }
    /// ```
    pub fn get_address<T: Sized>(&self, address: Cell) -> AmxResult<Box<T>> {
        unsafe {
            let header = (*self.amx).base as *const types::AMX_HEADER;
            
            let data = if (*self.amx).data.is_null() {
                (*self.amx).base as usize + (*header).dat as usize
            } else {
                (*self.amx).data as usize
            };

            if address >= (*self.amx).hea && address < (*self.amx).stk || address < 0 || address >= (*self.amx).stp {
                Err(AmxError::MemoryAccess)
            } else {
                Ok(Box::from_raw((data + address as usize) as *mut T))
            }
        }
    }

    pub fn get_address_experemental<'a, T: Sized>(&'a self, address: Cell) -> AmxResult<&'a mut T> {
        unsafe {
            let header = (*self.amx).base as *const types::AMX_HEADER;
            
            let data = if (*self.amx).data.is_null() {
                (*self.amx).base as usize + (*header).dat as usize
            } else {
                (*self.amx).data as usize
            };

            if address >= (*self.amx).hea && address < (*self.amx).stk || address < 0 || address >= (*self.amx).stp {
                Err(AmxError::MemoryAccess)
            } else {
                Ok(transmute(data + address as usize))
            }
        }
    }

    /// Push a primitive value to AMX stack
    ///
    /// # Examples
    ///
    /// ```
    /// let index = amx.find_public("OnPlayerHPChanged").unwrap();
    /// let player_id: u32 = 12;
    /// let health: f32 = 100.0;
    ///
    /// amx.push(health);
    /// amx.push(player_id);
    /// amx.exec(index);
    /// ```
    pub fn push<T: Sized>(&self, value: T) -> AmxResult<()> {
        let push = import!(Push);
        
        unsafe {
            call!(push(self.amx, transmute_copy(&value)) => ())
        }
    }

    /// Exec an AMX function.
    ///
    /// # Examples
    ///
    /// ```
    /// let index = amx.find_native("GetPlayerMoney").unwrap();
    /// amx.push(1); // a player with ID 1
    /// 
    /// match amx.exec(index) {
    ///     Ok(money) => log!("Player has {} money.", money),
    ///     Err(err) => log!("Error: {:?}", err),
    /// }
    /// ```
    pub fn exec(&self, index: i32) -> AmxResult<i64> {
        let exec = import!(Exec);

        let retval = -1;
        unsafe {
            call!(exec(self.amx, transmute(&retval), index) => retval)
        }
    }

    /// Return an index of a public by its name.
    ///
    /// # Examples
    ///
    /// ```
    /// let public_index = amx.find_public("OnPlayerConnect").unwrap();
    /// ```
    pub fn find_public(&self, name: &str) -> AmxResult<i32> {
        let find_public = import!(FindPublic);

        let index = -1;
        let c_name = CString::new(name).unwrap();
        
        unsafe {
            call!(find_public(self.amx, c_name.as_ptr(), transmute(&index)) => index)
        }
    }

    /// Return an index of a native by its name.
    ///
    /// # Examples
    /// See `find_public` and `exec` examples.
    pub fn find_native(&self, name: &str) -> AmxResult<i32> {
        let find_native = import!(FindNative);

        let index = -1;
        let c_name = CString::new(name).unwrap();
        
        unsafe {
            call!(find_native(self.amx, c_name.as_ptr(), transmute(&index)) => index)
        }
    }

    pub fn find_pubvar(&self, name: &str) -> AmxResult<i32> {
        let find_pubvar = import!(FindPubVar);

        let value = -1;
        let c_name = CString::new(name).unwrap();

        unsafe {
            call!(find_pubvar(self.amx, c_name.as_ptr(), transmute(&value)) => value)
        }
    }

    /// Get length of a string.
    pub fn string_len(&self, address: *const Cell) -> AmxResult<usize> {
        let string_len = import!(StrLen);
        let mut length = 0;
        
        call!(string_len(address, &mut length) => length as usize)
    }

    /// Get a string from AMX.
    /// 
    /// # Examples
    /// 
    /// ```
    /// fn raw_function(amx: AMX, params: *mut types::Cell) -> AmxResult<Cell> {
    ///     unsafe {
    ///         let ptr = std::ptr::read(params.offset(1));
    ///         let mut addr = try!(amx.get_address::<i32>(ptr)); // get a pointer from amx
    ///         let len = try!(amx.string_len(addr.as_mut())); // get string length in amx
    ///         let string = try!(amx.get_string(addr.as_mut(), len + 1)); // convert amx string to rust String
    ///        
    ///         log!("got string: {}", string);
    ///
    ///         std::mem::forget(addr);
    ///     }
    ///
    ///     0
    /// }
    /// ```
    pub fn get_string(&self, address: *const Cell, length: usize) -> AmxResult<String> {
        let get_string = import!(GetString);
        let mut buffer: Vec<u8> = vec![0; length];
        let ptr = buffer.as_mut_slice().as_mut_ptr();

        let result = get_string(ptr, address, 0, length);

        if result == 0 {
            CString::new(&buffer[0..length - 1])
                .map_err(|_| AmxError::Params)
                .and_then(|cstring| cstring.into_string().map_err(|_| AmxError::Params))
        } else {
            Err(AmxError::from(result))
        }
    }

    /// Raise an AMX error.
    pub fn raise_error(&self, error: AmxError) -> AmxResult<()> {
        let raise_error = import!(RaiseError);
        call!(raise_error(self.amx, error as i32) => ())
    }
}

/// Custom error type for AMX errors.
/// Can be casted from i32
///
/// # Examples
/// 
/// ```
/// let find_public: samp_sdk::types::FindPublic_t = ...;
/// let result = find_public(...);
/// return AmxError::from(result);
/// ```
#[derive(Debug)]
pub enum AmxError {
    Exit = 1,
    Assert = 2,
    StackError = 3,
    Bounds = 4,
    MemoryAccess = 5,
    InvalidInstruction = 6,
    StackLow = 7,
    HeapLow = 8,
    Callback = 9,
    Native = 10,
    Divide = 11,
    Sleep = 12,
    InvalidState = 13,
    Memory = 16,
    Format = 17,
    Version = 18,
    NotFound = 19,
    Index = 20,
    Debug = 21,
    Init = 22,
    UserData = 23,
    InitJit = 24,
    Params = 25,
    Domain = 26,
    General = 27,
    Unknown,
}

impl From<i32> for AmxError {
    fn from(val: i32) -> Self {
        match val {
            1 => AmxError::Exit,
            2 => AmxError::Assert,
            3 => AmxError::StackError,
            4 => AmxError::Bounds,
            5 => AmxError::MemoryAccess,
            6 => AmxError::InvalidInstruction,
            7 => AmxError::StackLow,
            8 => AmxError::HeapLow,
            9 => AmxError::Callback,
            10 => AmxError::Native,
            11 => AmxError::Divide,
            12 => AmxError::Sleep,
            13 => AmxError::InvalidState,
            16 => AmxError::Memory,
            17 => AmxError::Format,
            18 => AmxError::Version,
            19 => AmxError::NotFound,
            20 => AmxError::Index,
            21 => AmxError::Debug,
            22 => AmxError::Init,
            23 => AmxError::UserData,
            24 => AmxError::InitJit,
            25 => AmxError::Params,
            26 => AmxError::Domain,
            27 => AmxError::General,
            _ => AmxError::Unknown,
        }
    }
}