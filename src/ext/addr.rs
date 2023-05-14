use std::net::{IpAddr, Ipv4Addr};

pub trait Increment: Sized {
    fn increment(&mut self) -> Self;
}

impl Increment for IpAddr {
    fn increment(&mut self) -> Self {
        match self {
            IpAddr::V4(ipv4) => {
                let value = u32::from(*ipv4) + 1;
                *self = Self::from(Ipv4Addr::from(value));
                self.clone()
            },
            IpAddr::V6(_) => unimplemented!(),
        }
    }
}