# Nice Enum

A proc-macro that makes working with enums a little nicer.

## Usage

```rust
#[derive(NiceEnum)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}
```

Generates the following:

```rust
pub enum IpAddrKind {
    V4,
    V6
}

impl IpAddr {
    pub fn kind(&self) -> IpAddrKind {
        match self {
            Self::V4(_) => IpAddrKind::V4,
            Self::V6(_) => IpAddrKind::V6,
        }
    }

    pub fn is_v4(&self) -> bool {
        matches!(self.kind(), IpAddrKind::V4)
    }

    pub fn is_v6(&self) -> bool {
        matches!(self.kind(), IpAddrKind::V6)
    }

    pub fn as_v4(&self) -> Option<&Ipv4Addr> {
        match self {
            Self::V4(value) => Some(value),
            _ => None
        }
    }

    pub fn as_v6(&self) -> Option<&Ipv6Addr> {
        match self {
            Self::V6(value) => Some(value),
            _ => None
        }
    }

    pub fn as_v4_mut(&mut self) -> Option<&mut Ipv4Addr> {
        match self {
            Self::V4(value) => Some(value),
            _ => None
        }
    }

    pub fn as_v6_mut(&mut self) -> Option<&mut Ipv6Addr> {
        match self {
            Self::V6(value) => Some(value),
            _ => None
        }
    }

    pub fn into_v4(self) -> Option<Ipv4Addr> {
        match self {
            Self::V4(value) => Some(value),
            _ => None
        }
    }

    pub fn into_v6(self) -> Option<Ipv6Addr> {
        match self {
            Self::V6(value) => Some(value),
            _ => None
        }
    }
}
```
