use {
    crate::{GeoData, GeoIpResolver},
    std::{convert::Infallible, net::IpAddr},
};

#[derive(Debug, Clone)]
pub struct LocalResolver {
    resolver: fn(IpAddr) -> GeoData,
}

impl LocalResolver {
    pub fn new(resolver: fn(IpAddr) -> GeoData) -> Self {
        Self { resolver }
    }
}

impl GeoIpResolver for LocalResolver {
    type Error = Infallible;

    fn lookup_geo_data(&self, addr: IpAddr) -> Result<GeoData, Self::Error> {
        Ok((self.resolver)(addr))
    }
}
