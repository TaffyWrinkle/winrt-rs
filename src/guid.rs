use super::{AbiTransferable, RuntimeType};

/// A globally unique identifier [(GUID)](https://docs.microsoft.com/en-us/dotnet/api/system.guid?view=netcore-3.1)
/// used to uniquely identify COM and WinRT interfaces.
#[repr(C)]
#[derive(Clone, Default, PartialEq)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

impl Guid {
    /// Creates a `Guid` represented by the all-zero byte-pattern.
    pub const fn zeroed() -> Guid {
        Guid {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    /// Creates a `Guid` with the given constant values.
    pub const fn from_values(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Guid {
        Guid {
            data1,
            data2,
            data3,
            data4,
        }
    }

    /// Creates a `Guid` for a "generic" WinRT type.
    pub const fn from_signature(signature: crate::ConstString) -> Guid {
        let data = crate::ConstString::from_slice(&[
            0x11, 0xf4, 0x7a, 0xd5, 0x7b, 0x73, 0x42, 0xc0, 0xab, 0xae, 0x87, 0x8b, 0x1e, 0x16,
            0xad, 0xee,
        ]);

        let first = signature.get(0);
        let data = data.push_other(signature);

        let hash = const_sha1::sha1(&data).data;

        let second = hash[1].to_be_bytes();
        let mut data3 = u16::from_be_bytes([second[2], second[3]]);
        data3 = (data3 & 0x0fff) | (5 << 12);
        let third = hash[2].to_be_bytes();
        let fourth = hash[3].to_be_bytes();

        Self::from_values(
            first as u32,
            u16::from_be_bytes([second[0], second[1]]),
            data3,
            [
                ((third[0] & 0x3f) | 0x80) as u8,
                third[1],
                third[2],
                third[3],
                fourth[0],
                fourth[1],
                fourth[2],
                fourth[3],
            ],
        )
    }
}

unsafe impl AbiTransferable for Guid {
    type Abi = Self;

    fn get_abi(&self) -> Self::Abi {
        self.clone()
    }

    fn set_abi(&mut self) -> *mut Self::Abi {
        self as *mut Self::Abi
    }
}
unsafe impl RuntimeType for Guid {
    const SIGNATURE: crate::ConstString = crate::ConstString::from_slice(b"g16");
}

impl std::fmt::Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:08X?}-{:04X?}-{:04X?}-{:02X?}{:02X?}-{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

impl From<&str> for Guid {
    fn from(value: &str) -> Guid {
        assert!(value.len() == 36, "Invalid GUID string");
        let mut bytes = value.bytes();

        let a = ((bytes.next_u32() * 16 + bytes.next_u32()) << 24)
            + ((bytes.next_u32() * 16 + bytes.next_u32()) << 16)
            + ((bytes.next_u32() * 16 + bytes.next_u32()) << 8)
            + bytes.next_u32() * 16
            + bytes.next_u32();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let b = ((bytes.next_u16() * 16 + (bytes.next_u16())) << 8)
            + bytes.next_u16() * 16
            + bytes.next_u16();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let c = ((bytes.next_u16() * 16 + bytes.next_u16()) << 8)
            + bytes.next_u16() * 16
            + bytes.next_u16();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let d = bytes.next_u8() * 16 + bytes.next_u8();
        let e = bytes.next_u8() * 16 + bytes.next_u8();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");

        let f = bytes.next_u8() * 16 + bytes.next_u8();
        let g = bytes.next_u8() * 16 + bytes.next_u8();
        let h = bytes.next_u8() * 16 + bytes.next_u8();
        let i = bytes.next_u8() * 16 + bytes.next_u8();
        let j = bytes.next_u8() * 16 + bytes.next_u8();
        let k = bytes.next_u8() * 16 + bytes.next_u8();

        Guid::from_values(a, b, c, [d, e, f, g, h, i, j, k])
    }
}

trait HexReader {
    fn next_u8(&mut self) -> u8;
    fn next_u16(&mut self) -> u16;
    fn next_u32(&mut self) -> u32;
}

impl HexReader for std::str::Bytes<'_> {
    fn next_u8(&mut self) -> u8 {
        let value = self.next().unwrap();
        match value {
            b'0'..=b'9' => value - b'0',
            b'A'..=b'F' => 10 + value - b'A',
            b'a'..=b'f' => 10 + value - b'a',
            _ => panic!("Invalid GUID string"),
        }
    }

    fn next_u16(&mut self) -> u16 {
        self.next_u8().into()
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u8().into()
    }
}
